//! The components for using the implementor of `Endpoint` as an HTTP `Service`.

#![allow(missing_docs)]

use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;

use bytes::Bytes;
use either::Either;
use futures::{future, Async, Future, Poll};
use http::header::{self, HeaderValue};
use http::{Request, Response};
use izanami_service::{MakeService, Service};

use crate::endpoint::{
    ActionContext, //
    Endpoint,
    EndpointAction,
    IsEndpoint,
    Preflight,
    PreflightContext,
};
use crate::error::Error;
use crate::output::IntoResponse;

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
                    let mut acx = ActionContext::new(&mut self.request, &mut self.body);
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
        let output = ready!(self.poll_apply());

        let mut response = match output {
            Ok(output) => output.into_response(&self.request).map(Either::Right),
            Err(err) => err.into_response(&self.request).map(Either::Left),
        };

        response
            .headers_mut()
            .entry(header::SERVER)
            .unwrap()
            .or_insert_with(|| {
                HeaderValue::from_static(concat!("finchers/", env!("CARGO_PKG_VERSION")))
            });

        Ok(Async::Ready(response))
    }
}

pub type ResponseBody<Bd, E> = Either<Bytes, <<E as Endpoint<Bd>>::Output as IntoResponse>::Body>;
