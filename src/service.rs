//! Components of lower-level HTTP services

use std::mem;
use futures::{Future, IntoFuture, Poll};
use futures::Async::*;
use hyper::{Body, Error};
use hyper::server::Service;
use http_crate::{Request, Response};

use endpoint::{Endpoint, EndpointResult};
use http::IntoResponse;
use handler::{DefaultHandler, Handler};
use responder::{DefaultResponder, Responder};

/// An HTTP service which wraps a `Endpoint`, `Handler` and `Responder`.
#[derive(Debug)]
pub struct FinchersService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item, Error = E::Error> + Clone,
    R: Responder<H::Item, H::Error> + Clone,
{
    endpoint: E,
    handler: H,
    responder: R,
}

impl<E, H, R> FinchersService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item, Error = E::Error> + Clone,
    R: Responder<H::Item, H::Error> + Clone,
{
    /// Create an instance of `FinchersService` from components
    pub fn new(endpoint: E, handler: H, responder: R) -> Self {
        Self {
            endpoint,
            handler,
            responder,
        }
    }
}

impl<E, H, R> Copy for FinchersService<E, H, R>
where
    E: Endpoint + Copy,
    H: Handler<E::Item, Error = E::Error> + Copy,
    R: Responder<H::Item, H::Error> + Copy,
{
}

impl<E, H, R> Clone for FinchersService<E, H, R>
where
    E: Endpoint + Clone,
    H: Handler<E::Item, Error = E::Error> + Clone,
    R: Responder<H::Item, H::Error> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            handler: self.handler.clone(),
            responder: self.responder.clone(),
        }
    }
}

impl<E, H, R> Service for FinchersService<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item, Error = E::Error> + Clone,
    R: Responder<H::Item, H::Error> + Clone,
{
    type Request = Request<Body>;
    type Response = Response<Body>;
    type Error = Error;
    type Future = FinchersServiceFuture<E, H, R>;

    fn call(&self, request: Self::Request) -> Self::Future {
        FinchersServiceFuture {
            state: match self.endpoint.apply_request(request.map(Some)) {
                Some(input) => State::PollingInput {
                    input,
                    handler: self.handler.clone(),
                },
                None => State::NoRoute,
            },
            responder: self.responder.clone(),
        }
    }
}

/// A future returned from `EndpointService::call()`
#[allow(missing_debug_implementations)]
pub struct FinchersServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item, Error = E::Error>,
    R: Responder<H::Item, H::Error>,
{
    state: State<E, H>,
    responder: R,
}

#[allow(missing_debug_implementations)]
enum State<E, H>
where
    E: Endpoint,
    H: Handler<E::Item>,
{
    NoRoute,
    PollingInput {
        input: <E::Result as EndpointResult>::Future,
        handler: H,
    },
    PollingOutput {
        output: <H::Result as IntoFuture>::Future,
    },
    Done,
}

impl<E, H, R> FinchersServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item, Error = E::Error>,
    R: Responder<H::Item, H::Error>,
{
    fn poll_state(&mut self) -> Poll<Result<Option<H::Item>, H::Error>, Error> {
        use self::State::*;
        loop {
            match mem::replace(&mut self.state, Done) {
                NoRoute => break Ok(Ready(Ok(None))),
                PollingInput { mut input, handler } => match input.poll() {
                    Ok(Ready(input)) => {
                        self.state = PollingOutput {
                            output: IntoFuture::into_future(handler.call(input)),
                        };
                        continue;
                    }
                    Ok(NotReady) => {
                        self.state = PollingInput { input, handler };
                        break Ok(NotReady);
                    }
                    Err(Ok(err)) => break Ok(Ready(Err(err))),
                    Err(Err(err)) => break Err(err),
                },
                PollingOutput { mut output } => match output.poll() {
                    Ok(Ready(item)) => break Ok(Ready(Ok(item))),
                    Ok(NotReady) => {
                        self.state = PollingOutput { output };
                        break Ok(NotReady);
                    }
                    Err(err) => break Ok(Ready(Err(err))),
                },
                Done => panic!(),
            }
        }
    }
}

impl<E, H, R> Future for FinchersServiceFuture<E, H, R>
where
    E: Endpoint,
    H: Handler<E::Item, Error = E::Error>,
    R: Responder<H::Item, H::Error>,
{
    type Item = Response<Body>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let response = match try_ready!(self.poll_state()) {
            Ok(Some(item)) => self.responder.respond_ok(item),
            Ok(None) => self.responder.respond_noroute(),
            Err(err) => self.responder.respond_err(err),
        };
        let mut response = response.unwrap_or_else(|e| {
            use http_crate::{Response, StatusCode};
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("server_error: {}", e).into())
                .expect("failed to construct an error response")
        });
        self.responder.after_respond(&mut response);
        Ok(Ready(response))
    }
}

#[allow(missing_docs)]
pub trait EndpointServiceExt: Endpoint + sealed::Sealed
where
    Self::Item: IntoResponse,
    Self::Error: IntoResponse,
{
    fn into_service(self) -> FinchersService<Self, DefaultHandler<Self::Error>, DefaultResponder>
    where
        Self: Sized;

    fn with_handler<H>(self, handler: H) -> FinchersService<Self, H, DefaultResponder>
    where
        H: Handler<Self::Item, Error = Self::Error> + Clone,
        H::Item: IntoResponse,
        H::Error: IntoResponse,
        Self: Sized;
}

impl<E: Endpoint> EndpointServiceExt for E
where
    E::Item: IntoResponse,
    E::Error: IntoResponse,
{
    fn into_service(self) -> FinchersService<Self, DefaultHandler<Self::Error>, DefaultResponder> {
        FinchersService::new(self, DefaultHandler::default(), Default::default())
    }

    fn with_handler<H>(self, handler: H) -> FinchersService<Self, H, DefaultResponder>
    where
        H: Handler<Self::Item, Error = Self::Error> + Clone,
        H::Item: IntoResponse,
        H::Error: IntoResponse,
    {
        FinchersService::new(self, handler, Default::default())
    }
}

mod sealed {
    use endpoint::Endpoint;
    pub trait Sealed {}
    impl<E: Endpoint> Sealed for E {}
}

mod tests {
    #[test]
    fn smoke_service_ext() {
        use endpoint::prelude::*;
        use std::rc::Rc;
        use super::EndpointServiceExt;

        let endpoint = endpoint("foo").assert_types::<(), ()>();
        let _ = endpoint.clone().into_service();
        let _ = endpoint.with_handler(Rc::new(|()| Ok(Some("Hello"))));
    }
}
