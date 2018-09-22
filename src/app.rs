//! The components to construct an asynchronous HTTP service from the `Endpoint`.

use futures::{Async, Future, Poll};
use std::io;

use http::header::HeaderValue;
use http::{header, Request, Response};

use crate::common::Either;
use crate::endpoint::{Context, Endpoint};
use crate::error::Error;
use crate::input::body::ReqBody;
use crate::input::{with_set_cx, Input};
use crate::output::payload::Once;
use crate::output::{Output, OutputContext};

/// A factory of HTTP service which wraps an `Endpoint`.
#[derive(Debug)]
pub struct App<'e, E: Endpoint<'e>> {
    endpoint: &'e E,
}

impl<'e, E: Endpoint<'e>> App<'e, E> {
    /// Create a new `App` from the provided components.
    pub fn new(endpoint: &'e E) -> App<'e, E> {
        App { endpoint }
    }

    #[allow(missing_docs)]
    pub fn dispatch_request(&self, request: Request<ReqBody>) -> AppFuture<'e, E> {
        AppFuture {
            state: State::Uninitialized,
            input: Input::new(request),
            endpoint: self.endpoint,
        }
    }
}

pub type ResBody<T> = Either<Once<String>, <T as Output>::Body>;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AppFuture<'e, E: Endpoint<'e>> {
    state: State<E::Future>,
    input: Input,
    endpoint: &'e E,
}

#[derive(Debug)]
enum State<T> {
    Uninitialized,
    InFlight(T),
    Gone,
}

impl<'e, E> AppFuture<'e, E>
where
    E: Endpoint<'e>,
{
    pub fn poll_output(&mut self) -> Poll<E::Output, Error> {
        loop {
            match self.state {
                State::Uninitialized => {
                    let mut ecx = Context::new(&mut self.input);
                    match self.endpoint.apply(&mut ecx) {
                        Ok(future) => self.state = State::InFlight(future),
                        Err(err) => {
                            self.state = State::Gone;
                            return Err(err.into());
                        }
                    }
                }
                State::InFlight(ref mut f) => {
                    break with_set_cx(&mut self.input, || f.poll());
                }
                State::Gone => panic!("cannot poll AppServiceFuture twice"),
            }
        }
    }

    pub fn poll_response(&mut self) -> Poll<Response<ResBody<E::Output>>, io::Error>
    where
        E::Output: Output,
    {
        let output = match self.poll_output() {
            Ok(Async::Ready(item)) => Ok(item),
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(err) => Err(err),
        };

        let mut response = output
            .and_then({
                let mut cx = OutputContext::new(&mut self.input);
                move |out| {
                    out.respond(&mut cx)
                        .map(|res| res.map(Either::Right))
                        .map_err(Into::into)
                }
            }).unwrap_or_else(|err| err.to_response().map(|body| Either::Left(Once::new(body))));

        if let Some(jar) = self.input.cookie_jar() {
            for cookie in jar.delta() {
                let val = HeaderValue::from_str(&cookie.encoded().to_string()).unwrap();
                response.headers_mut().insert(header::SET_COOKIE, val);
            }
        }

        response
            .headers_mut()
            .entry(header::SERVER)
            .unwrap()
            .or_insert(HeaderValue::from_static(concat!(
                "finchers-runtime/",
                env!("CARGO_PKG_VERSION")
            )));

        Ok(Async::Ready(response))
    }
}

impl<'e, E> Future for AppFuture<'e, E>
where
    E: Endpoint<'e>,
    E::Output: Output,
{
    type Item = Response<ResBody<E::Output>>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.poll_response()
    }
}

mod service {
    use super::{App, AppFuture, ResBody};

    use std::io;

    use futures as futures01;
    use http::Request;
    use hyper::body::Body;
    use hyper::service::{NewService, Service};

    use crate::endpoint::Endpoint;
    use crate::input::body::ReqBody;
    use crate::output::Output;

    impl<'e, E: Endpoint<'e>> NewService for App<'e, E>
    where
        E::Output: Output,
    {
        type ReqBody = Body;
        type ResBody = ResBody<E::Output>;
        type Error = io::Error;
        type Service = Self;
        type InitError = io::Error;
        type Future = futures01::future::FutureResult<Self::Service, Self::InitError>;

        fn new_service(&self) -> Self::Future {
            futures01::future::ok(App {
                endpoint: self.endpoint,
            })
        }
    }

    impl<'e, E: Endpoint<'e>> Service for App<'e, E>
    where
        E::Output: Output,
    {
        type ReqBody = Body;
        type ResBody = ResBody<E::Output>;
        type Error = io::Error;
        type Future = AppFuture<'e, E>;

        fn call(&mut self, request: Request<Self::ReqBody>) -> Self::Future {
            self.dispatch_request(request.map(ReqBody::from_hyp))
        }
    }
}
