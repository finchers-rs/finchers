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
    std::{cell::Cell, io, marker::PhantomData, ptr::NonNull, sync::Arc},
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

#[doc(hidden)]
#[allow(missing_debug_implementations)]
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
            state: AppFutureState::Start(Some(self.endpoint.action())),
            context: Context::new(Request::from_parts(parts, ())),
            body: Some(body),
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

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct AppFuture<Bd, E: Endpoint<Bd>> {
    state: AppFutureState<E::Action>,
    context: Context,
    body: Option<Bd>,
}

#[allow(missing_debug_implementations, clippy::large_enum_variant)]
enum AppFutureState<A> {
    Start(Option<A>),
    InFlight(A),
}

impl<Bd, E> AppFuture<Bd, E>
where
    E: Endpoint<Bd>,
{
    pub(crate) fn poll_apply(&mut self) -> Poll<E::Output, Error> {
        loop {
            self.state = match self.state {
                AppFutureState::Start(ref mut action) => {
                    let mut action = action.take().unwrap();
                    let mut ecx = PreflightContext::new(&self.context);
                    if let Preflight::Completed(output) = action.preflight(&mut ecx)? {
                        return Ok(Async::Ready(output));
                    }
                    AppFutureState::InFlight(action)
                }
                AppFutureState::InFlight(ref mut action) => {
                    return action.poll_action(&mut ActionContext::new(
                        &mut self.context, //
                        &mut self.body,
                    ));
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
                .into_response(&self.context.request)
                .map(izanami_util::buf_stream::Either::Right),
            Err(err) => err
                .into_response(&self.context.request)
                .map(izanami_util::buf_stream::Either::Left),
        };

        if let Some(cookies) = &self.context.cookies {
            for cookie in cookies.delta() {
                response
                    .headers_mut()
                    .append(http::header::SET_COOKIE, encode_cookie(cookie));
            }
        }

        if let Some(mut hdrs) = self.context.response_headers.take() {
            for (name, values) in hdrs.drain() {
                response
                    .headers_mut()
                    .extend(values.map(|value| (name.clone(), value)));
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

// ==== Context ====

thread_local! {
    static TLS_CX: Cell<Option<NonNull<Context>>> = Cell::new(None);
}

#[allow(missing_debug_implementations)]
struct ResetOnDrop(Option<NonNull<Context>>);

impl Drop for ResetOnDrop {
    fn drop(&mut self) {
        TLS_CX.with(|tls_cx| {
            tls_cx.set(self.0.take());
        })
    }
}

fn is_set_task_cx() -> bool {
    TLS_CX.with(|tls_cx| tls_cx.get().is_some())
}

fn set_task_cx<R>(cx: &mut Context, f: impl FnOnce() -> R) -> R {
    let old_cx = TLS_CX.with(|tls_cx| tls_cx.replace(Some(NonNull::from(cx))));
    let _reset = ResetOnDrop(old_cx);
    f()
}

fn get_task_cx<R>(f: impl FnOnce(&mut Context) -> R) -> Option<R> {
    let cx_ptr = TLS_CX.with(|tls_cx| tls_cx.replace(None));
    let _reset = ResetOnDrop(cx_ptr);
    cx_ptr.map(|mut cx_ptr| unsafe { f(cx_ptr.as_mut()) })
}

/// A set of miscellaneous context values used within a request handling.
#[derive(Debug)]
pub struct Context {
    request: Request<()>,
    cookies: Option<CookieJar>,
    response_headers: Option<HeaderMap>,
}

impl Context {
    pub(crate) fn new(request: Request<()>) -> Self {
        Context {
            request,
            cookies: None,
            response_headers: None,
        }
    }

    #[inline]
    pub fn set<R>(&mut self, f: impl FnOnce() -> R) -> R {
        set_task_cx(self, f)
    }

    #[inline]
    pub fn with<R>(f: impl FnOnce(&mut Self) -> R) -> R {
        get_task_cx(f).expect("TLS context is not set")
    }

    #[inline]
    pub fn try_with<R>(f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        get_task_cx(f)
    }

    #[inline]
    pub fn is_set() -> bool {
        is_set_task_cx()
    }

    /// Returns a reference to the inner `Request<()>`.
    pub fn request(&self) -> &Request<()> {
        &self.request
    }

    /// Returns a mutable reference to the inner `Request<()>`.
    pub fn request_mut(&mut self) -> &mut Request<()> {
        &mut self.request
    }

    /// Initializes the inner `CookieJar` and returns a mutable reference to its instance.
    pub fn cookies(&mut self) -> Result<&mut CookieJar, Error> {
        if let Some(ref mut cookies) = self.cookies {
            Ok(cookies)
        } else {
            let cookies = self.cookies.get_or_insert_with(CookieJar::new);
            for raw_cookie in self.request.headers().get_all(http::header::COOKIE) {
                let raw_cookie_str = raw_cookie.to_str().map_err(crate::error::bad_request)?;
                for s in raw_cookie_str.split(';').map(|s| s.trim()) {
                    let cookie = Cookie::parse_encoded(s)
                        .map_err(crate::error::bad_request)?
                        .into_owned();
                    cookies.add_original(cookie);
                }
            }
            Ok(cookies)
        }
    }

    /// Returns a mutable reference to a `HeaderMap` which contains the supplemental response headers.
    pub fn response_headers(&mut self) -> &mut HeaderMap {
        self.response_headers.get_or_insert_with(Default::default)
    }
}

impl std::ops::Deref for Context {
    type Target = Request<()>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.request()
    }
}

impl std::ops::DerefMut for Context {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.request_mut()
    }
}
