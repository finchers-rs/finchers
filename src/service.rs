//! The components for using the implementor of `Endpoint` as an HTTP `Service`.

#![allow(missing_docs)]

use {
    crate::{
        action::{ActionContext, EndpointAction, Preflight, PreflightContext},
        endpoint::{Endpoint, IsEndpoint},
        error::Error,
        output::IntoResponse,
    },
    bytes::{BufMut, BytesMut},
    cookie::{Cookie, CookieJar},
    futures::{future, Async, Future, Poll},
    http::{
        header::{HeaderMap, HeaderValue},
        Request, Response,
    },
    izanami_service::{MakeService, Service},
    std::{fmt, io, marker::PhantomData, sync::Arc},
};

macro_rules! ready {
    ($e:expr) => {
        match $e {
            Ok(Async::Ready(ok)) => Ok(ok),
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(err) => Err(err),
        }
    };
}

pub trait EndpointServiceExt: IsEndpoint + Sized {
    fn into_service(self) -> App<Self>;
}

impl<E: IsEndpoint> EndpointServiceExt for E {
    fn into_service(self) -> App<Self> {
        App::new(self)
    }
}

/// A wrapper struct for lifting the instance of `Endpoint` to an HTTP service.
#[derive(Debug)]
pub struct App<E> {
    endpoint: Arc<E>,
}

impl<E> App<E> {
    /// Create a new `App` from the specified endpoint.
    pub fn new(endpoint: E) -> Self {
        App {
            endpoint: Arc::new(endpoint),
        }
    }
}

impl<E, Ctx, Bd> MakeService<Ctx, Request<Bd>> for App<E>
where
    E: Endpoint<Bd>,
    E::Output: IntoResponse,
{
    type Response = Response<ResponseBody<Bd, E>>;
    type Error = io::Error;
    type Service = AppService<Bd, Arc<E>>;
    type MakeError = io::Error;
    type Future = future::FutureResult<Self::Service, Self::MakeError>;

    fn make_service(&self, _: Ctx) -> Self::Future {
        future::ok(AppService::new(self.endpoint.clone()))
    }
}

#[derive(Debug)]
pub struct AppService<Bd, E: Endpoint<Bd>> {
    endpoint: E,
    _marker: PhantomData<fn(Bd)>,
}

impl<Bd, E> AppService<Bd, E>
where
    E: Endpoint<Bd>,
{
    pub(crate) fn new(endpoint: E) -> Self {
        AppService {
            endpoint,
            _marker: PhantomData,
        }
    }

    pub(crate) fn dispatch(&self, request: Request<Bd>) -> AppFuture<Bd, E> {
        let (parts, body) = request.into_parts();
        AppFuture {
            state: State::Start(Some(self.endpoint.action())),
            request: Request::from_parts(parts, ()),
            body: Some(body),
            cookies: None,
            response_headers: None,
        }
    }
}

impl<Bd, E> Service<Request<Bd>> for AppService<Bd, E>
where
    E: Endpoint<Bd> + Clone,
    E::Output: IntoResponse,
{
    type Response = Response<ResponseBody<Bd, E>>;
    type Error = io::Error;
    type Future = AppFuture<Bd, E>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, request: Request<Bd>) -> Self::Future {
        self.dispatch(request)
    }
}

pub struct AppFuture<Bd, E: Endpoint<Bd>> {
    state: State<E::Action>,
    request: Request<()>,
    body: Option<Bd>,
    cookies: Option<CookieJar>,
    response_headers: Option<HeaderMap>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum State<A> {
    Start(Option<A>),
    InFlight(A),
}

impl<Bd, E> fmt::Debug for AppFuture<Bd, E>
where
    E: Endpoint<Bd>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppFuture").finish()
    }
}

impl<Bd, E> AppFuture<Bd, E>
where
    E: Endpoint<Bd>,
{
    pub(crate) fn poll_apply(&mut self) -> Poll<E::Output, Error> {
        loop {
            self.state = match self.state {
                State::Start(ref mut action) => {
                    let mut action = action.take().unwrap();
                    let mut ecx = PreflightContext::new(&self.request);
                    if let Preflight::Completed(output) = action.preflight(&mut ecx)? {
                        return Ok(Async::Ready(output));
                    }
                    State::InFlight(action)
                }
                State::InFlight(ref mut action) => {
                    let mut acx = ActionContext::new(
                        &mut self.request, //
                        &mut self.body,
                        &mut self.cookies,
                        &mut self.response_headers,
                    );
                    return action.poll_action(&mut acx);
                }
            };
        }
    }
}

impl<Bd, E> Future for AppFuture<Bd, E>
where
    E: Endpoint<Bd>,
    E::Output: IntoResponse,
{
    type Item = Response<ResponseBody<Bd, E>>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut response = match ready!(self.poll_apply()) {
            Ok(output) => output
                .into_response(&self.request)
                .map(izanami_util::buf_stream::Either::Right),
            Err(err) => err
                .into_response(&self.request)
                .map(izanami_util::buf_stream::Either::Left),
        };

        if let Some(cookies) = &self.cookies {
            for cookie in cookies.delta() {
                response
                    .headers_mut()
                    .append(http::header::SET_COOKIE, encode_cookie(cookie));
            }
        }

        if let Some(mut hdrs) = self.response_headers.take() {
            for (name, values) in hdrs.drain() {
                response.headers_mut().extend(
                    values //
                        .into_iter()
                        .map(|value| (name.clone(), value)),
                );
            }
        }

        Ok(Async::Ready(response))
    }
}

pub type ResponseBody<Bd, E> = izanami_util::buf_stream::Either<
    String, //
    <<E as Endpoint<Bd>>::Output as IntoResponse>::Body,
>;

/// Encode a Cookie value into a `HeaderValue`
fn encode_cookie(cookie: &Cookie<'_>) -> HeaderValue {
    use std::io::Write;

    let estimated_capacity = cookie.name().len() + cookie.value().len() + 1; // name=value
    let mut value = BytesMut::with_capacity(estimated_capacity);
    let _ = write!((&mut value).writer(), "{}", cookie.encoded());

    // safety: the bytes genereted by EncodedCookie is a valid header value.
    unsafe { HeaderValue::from_shared_unchecked(value.freeze()) }
}
