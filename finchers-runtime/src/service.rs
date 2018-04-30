//! Components of lower-level HTTP services

use futures::Async::*;
use futures::{Future, Poll};
use http::StatusCode;
use http::header::{self, HeaderValue};
use http::{Request, Response};
use std::sync::Arc;
use std::{fmt, io};

use finchers_core::endpoint::{ApplyRequest, PollReady};
use finchers_core::error::ServerError;
use finchers_core::input::RequestBody;
use finchers_core::output::{Body, Responder};
use finchers_core::{Endpoint, HttpError, Input, Task};

#[allow(missing_docs)]
pub trait HttpService {
    type RequestBody;
    type ResponseBody;
    type Error;
    type Future: Future<Item = Response<Self::ResponseBody>, Error = Self::Error>;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future;
}

impl<S: HttpService> HttpService for Box<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future {
        (**self).call(request)
    }
}

impl<S: HttpService> HttpService for Arc<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future {
        (**self).call(request)
    }
}

/// An HTTP service which wraps an `Endpoint`.
pub struct EndpointService<E> {
    endpoint: E,
    error_handler: ErrorHandler,
}

impl<E> EndpointService<E> {
    pub fn new(endpoint: E) -> EndpointService<E> {
        Self {
            endpoint,
            error_handler: default_error_handler,
        }
    }

    pub fn set_error_handler(&mut self, handler: ErrorHandler) {
        self.error_handler = handler;
    }
}

impl<E> HttpService for EndpointService<E>
where
    E: Endpoint,
    E::Output: Responder,
{
    type RequestBody = RequestBody;
    type ResponseBody = Body;
    type Error = io::Error;
    type Future = EndpointServiceFuture<E::Task>;

    fn call(&self, request: Request<Self::RequestBody>) -> Self::Future {
        let (parts, body) = request.into_parts();
        let input = Input::new(Request::from_parts(parts, ()));
        let apply = self.endpoint.apply_request(&input, body);

        EndpointServiceFuture {
            apply,
            input,
            error_handler: self.error_handler,
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct EndpointServiceFuture<T> {
    apply: ApplyRequest<T>,
    input: Input,
    error_handler: ErrorHandler,
}

impl<T> EndpointServiceFuture<T> {
    fn handle_error(&self, err: &HttpError) -> Response<Body> {
        (self.error_handler)(err, &self.input)
    }
}

impl<T> Future for EndpointServiceFuture<T>
where
    T: Task,
    T::Output: Responder,
{
    type Item = Response<Body>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut response = match self.apply.poll_ready(&self.input) {
            PollReady::Pending => return Ok(NotReady),
            PollReady::Ready(Some(Ok(output))) => output
                .respond(&self.input)
                .unwrap_or_else(|err| self.handle_error(&ServerError::from_fail(err))),
            PollReady::Ready(Some(Err(err))) => self.handle_error(&*err.http_error()),
            PollReady::Ready(None) => self.handle_error(&NoRoute),
        };

        if !response.headers().contains_key(header::SERVER) {
            response.headers_mut().insert(
                header::SERVER,
                HeaderValue::from_static(concat!("finchers-runtime/", env!("CARGO_PKG_VERSION"))),
            );
        }

        Ok(Ready(response))
    }
}

#[derive(Debug)]
struct NoRoute;

impl fmt::Display for NoRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("no route")
    }
}

impl HttpError for NoRoute {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

///
pub type ErrorHandler = fn(&HttpError, &Input) -> Response<Body>;

fn default_error_handler(err: &HttpError, input: &Input) -> Response<Body> {
    let mut response = err.to_response(input).unwrap_or_else(|| {
        let body = err.to_string();
        let body_len = body.len().to_string();

        let mut response = Response::new(Body::once(body));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response.headers_mut().insert(header::CONTENT_LENGTH, unsafe {
            HeaderValue::from_shared_unchecked(body_len.into())
        });
        response
    });
    *response.status_mut() = err.status_code();
    response
}
