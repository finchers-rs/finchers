//! The components for using the implementor of `Endpoint` as an HTTP `Service`.

#![allow(missing_docs)]

use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

use either::Either;
use futures::future;
use futures::{Async, Future, Poll};
use http::header;
use http::header::HeaderValue;
use http::{Request, Response};
use izanami_service::{MakeService, Service};

use crate::endpoint::context::ApplyContext;
use crate::endpoint::{Cursor, Endpoint, IsEndpoint};
use crate::error::Error;
use crate::error::Never;
use crate::future::{Context, EndpointFuture};
use crate::input::Input;
use crate::output::body::ResBody;
use crate::output::IntoResponse;

pub trait EndpointServiceExt: IsEndpoint + Sized {
    fn into_service(self) -> App<Self>;
}

impl<E: IsEndpoint> EndpointServiceExt for E {
    fn into_service(self) -> App<Self> {
        App::new(self)
    }
}

pub type ResponseBody<Bd, E> = Either<
    String, //
    <<<E as Endpoint<Bd>>::Output as IntoResponse>::Body as ResBody>::Payload,
>;

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
    <E::Output as IntoResponse>::Body: ResBody,
{
    type Response = Response<ResponseBody<Bd, E>>;
    type Error = io::Error;
    type Service = AppService<Bd, Arc<E>>;
    type MakeError = Never;
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
    E: Endpoint<Bd> + Clone,
{
    pub(crate) fn new(endpoint: E) -> Self {
        AppService {
            endpoint,
            _marker: PhantomData,
        }
    }

    pub(crate) fn dispatch(&self, request: Request<Bd>) -> AppFuture<Bd, E> {
        AppFuture {
            endpoint: self.endpoint.clone(),
            state: State::Start(request),
        }
    }
}

impl<Bd, E> Service<Request<Bd>> for AppService<Bd, E>
where
    E: Endpoint<Bd> + Clone,
    E::Output: IntoResponse,
    <E::Output as IntoResponse>::Body: ResBody,
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

#[derive(Debug)]
pub struct AppFuture<Bd, E: Endpoint<Bd>> {
    endpoint: E,
    state: State<Bd, E>,
}

#[allow(clippy::large_enum_variant)]
enum State<Bd, E: Endpoint<Bd>> {
    Start(Request<Bd>),
    InFlight(Input<Bd>, E::Future, Cursor),
    Done(Input<Bd>),
    Gone,
}

impl<Bd, E> fmt::Debug for State<Bd, E>
where
    Bd: fmt::Debug,
    E: Endpoint<Bd>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Start(ref request) => f.debug_struct("Start").field("request", request).finish(),
            State::InFlight(ref input, _, ref cursor) => f
                .debug_struct("InFlight")
                .field("input", input)
                .field("cursor", cursor)
                .finish(),
            State::Done(ref input) => f.debug_struct("Done").field("input", input).finish(),
            State::Gone => f.debug_struct("Gone").finish(),
        }
    }
}

impl<Bd, E> AppFuture<Bd, E>
where
    E: Endpoint<Bd>,
{
    pub(crate) fn poll_apply(&mut self) -> Poll<E::Output, Error> {
        loop {
            let result = match self.state {
                State::Start(..) => None,
                State::InFlight(ref mut input, ref mut f, ref mut cursor) => {
                    let mut tcx = Context::new(input, cursor);
                    match f.poll_endpoint(&mut tcx) {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(ok)) => Some(Ok(ok)),
                        Err(err) => Some(Err(err)),
                    }
                }
                State::Done(..) | State::Gone => panic!("cannot poll AppServiceFuture twice"),
            };

            match (mem::replace(&mut self.state, State::Gone), result) {
                (State::Start(request), None) => {
                    let mut input = Input::new(request);
                    let mut cursor = Cursor::default();
                    match {
                        let mut ecx = ApplyContext::new(&mut input, &mut cursor);
                        self.endpoint.apply(&mut ecx)
                    } {
                        Ok(future) => self.state = State::InFlight(input, future, cursor),
                        Err(err) => {
                            self.state = State::Done(input);
                            return Err(err.into());
                        }
                    }
                }
                (State::InFlight(input, ..), Some(result)) => {
                    self.state = State::Done(input);
                    return result.map(Async::Ready);
                }
                _ => unreachable!("unexpected state"),
            }
        }
    }

    pub(crate) fn poll_all(&mut self) -> Poll<Response<ResponseBody<Bd, E>>, io::Error>
    where
        E::Output: IntoResponse,
        <E::Output as IntoResponse>::Body: ResBody,
    {
        let output = match self.poll_apply() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Ok(Async::Ready(output)) => Ok(output),
            Err(err) => Err(err),
        };

        match mem::replace(&mut self.state, State::Gone) {
            State::Done(input) => {
                let mut response = match output {
                    Ok(output) => output
                        .into_response(input.request())
                        .map(|bd| Either::Right(bd.into_payload())),
                    Err(err) => err.into_response().map(Either::Left),
                };

                if let Some(hdrs) = input.response_headers {
                    response.headers_mut().extend(hdrs);
                }

                response
                    .headers_mut()
                    .entry(header::SERVER)
                    .unwrap()
                    .or_insert_with(|| {
                        HeaderValue::from_static(concat!("finchers/", env!("CARGO_PKG_VERSION")))
                    });
                Ok(Async::Ready(response))
            }
            _ => unreachable!("unexpected condition"),
        }
    }
}

impl<Bd, E> Future for AppFuture<Bd, E>
where
    E: Endpoint<Bd>,
    E::Output: IntoResponse,
    <E::Output as IntoResponse>::Body: ResBody,
{
    type Item = Response<ResponseBody<Bd, E>>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.poll_all()
    }
}
