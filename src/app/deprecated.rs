//! The components to construct an asynchronous HTTP service from the `Endpoint`.

#![deprecated(since = "0.12.3")]

use futures::{Async, Future, Poll};
use std::io;

use either::Either;
use http::header;
use http::header::HeaderValue;
use http::{Request, Response};

use endpoint::{with_set_cx, ApplyContext, Cursor, Endpoint, TaskContext};
use error::Error;
use input::Input;
use input::ReqBody;
use output::body::ResBody;
use output::{Output, OutputContext};

/// A factory of HTTP service which wraps an `Endpoint`.
#[derive(Debug)]
pub(crate) struct App<'e, E: Endpoint<'e>> {
    endpoint: &'e E,
}

impl<'e, E: Endpoint<'e>> App<'e, E> {
    /// Create a new `App` from the provided components.
    pub(crate) fn new(endpoint: &'e E) -> App<'e, E> {
        App { endpoint }
    }

    #[allow(missing_docs)]
    pub(crate) fn dispatch_request(&self, request: Request<ReqBody>) -> AppFuture<'e, E> {
        AppFuture {
            state: State::Uninitialized,
            input: Input::new(request),
            endpoint: self.endpoint,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub(crate) struct AppFuture<'e, E: Endpoint<'e>> {
    state: State<E::Future>,
    input: Input,
    endpoint: &'e E,
}

#[derive(Debug)]
enum State<T> {
    Uninitialized,
    InFlight(T, Cursor),
    Gone,
}

impl<'e, E> AppFuture<'e, E>
where
    E: Endpoint<'e>,
{
    pub(crate) fn poll_output(&mut self) -> Poll<E::Output, Error> {
        loop {
            match self.state {
                State::Uninitialized => {
                    let mut cursor = Cursor::default();
                    match {
                        let mut ecx = ApplyContext::new(&mut self.input, &mut cursor);
                        self.endpoint.apply(&mut ecx)
                    } {
                        Ok(future) => self.state = State::InFlight(future, cursor),
                        Err(err) => {
                            self.state = State::Gone;
                            return Err(err.into());
                        }
                    }
                }
                State::InFlight(ref mut f, ref mut cursor) => {
                    let mut tcx = TaskContext::new(&mut self.input, cursor);
                    break with_set_cx(&mut tcx, || f.poll());
                }
                State::Gone => panic!("cannot poll AppServiceFuture twice"),
            }
        }
    }

    pub(crate) fn poll_response<Bd>(&mut self) -> Poll<Response<Either<String, Bd>>, io::Error>
    where
        E::Output: Output<Body = Bd>,
        Bd: ResBody,
    {
        let output = match self.poll_output() {
            Ok(Async::Ready(out)) => {
                let mut cx = OutputContext::new(&mut self.input);
                out.respond(&mut cx).map_err(Into::into)
            }
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(err) => Err(err),
        };

        let mut response = output
            .map(|response| response.map(Either::Right))
            .unwrap_or_else(|err| err.to_response().map(Either::Left));

        if let Some(jar) = self.input.cookie_jar() {
            for cookie in jar.delta() {
                let val = HeaderValue::from_str(&cookie.encoded().to_string()).unwrap();
                response.headers_mut().insert(header::SET_COOKIE, val);
            }
        }

        if let Some(headers) = self.input.take_response_headers() {
            response.headers_mut().extend(headers);
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

mod service {
    use super::{App, AppFuture};

    use std::io;

    use either::Either;
    use futures::future;
    use futures::{Future, Poll};
    use http::{Request, Response};
    use hyper::body::Body;
    use hyper::service::{NewService, Service};

    use endpoint::Endpoint;
    use input::ReqBody;
    use output::body::ResBody;
    use output::Output;

    impl<'e, E: Endpoint<'e>, Bd> NewService for App<'e, E>
    where
        E::Output: Output<Body = Bd>,
        Bd: ResBody,
    {
        type ReqBody = Body;
        type ResBody = <Either<String, Bd> as ResBody>::Payload;
        type Error = io::Error;
        type Service = Self;
        type InitError = io::Error;
        type Future = future::FutureResult<Self::Service, Self::InitError>;

        fn new_service(&self) -> Self::Future {
            future::ok(App {
                endpoint: self.endpoint,
            })
        }
    }

    impl<'e, E: Endpoint<'e>, Bd> Service for App<'e, E>
    where
        E::Output: Output<Body = Bd>,
        Bd: ResBody,
    {
        type ReqBody = Body;
        type ResBody = <Either<String, Bd> as ResBody>::Payload;
        type Error = io::Error;
        type Future = AppFuture<'e, E>;

        fn call(&mut self, request: Request<Self::ReqBody>) -> Self::Future {
            self.dispatch_request(request.map(ReqBody::new))
        }
    }

    impl<'e, E, Bd> Future for AppFuture<'e, E>
    where
        E: Endpoint<'e>,
        E::Output: Output<Body = Bd>,
        Bd: ResBody,
    {
        type Item = Response<<Either<String, Bd> as ResBody>::Payload>;
        type Error = io::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            self.poll_response()
                .map(|x| x.map(|res| res.map(|bd| bd.into_payload())))
        }
    }
}
