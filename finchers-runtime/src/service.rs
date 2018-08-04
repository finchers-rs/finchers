//! Abstractions for constructing HTTP services.
//!
//! The traits in this module have some compatiblity with the traits which will be provided
//! by Tower and Hyper in the future.

use futures::{self, Future};
use http::{Request, Response};
use std::sync::Arc;

/// Trait representing the streaming body of HTTP response.
pub trait Payload {
    /// A single chunk of the message body.
    type Data: AsRef<[u8]> + 'static;

    /// The error which will be returned from `poll_data`.
    type Error;

    /// Poll a `Data` from this stream.
    fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error>;
}

/// A factory of an asynchronous HTTP service.
pub trait NewHttpService {
    /// The type of message body in the request.
    type RequestBody;

    /// The type of message body in the response.
    type ResponseBody;

    /// The type of error which will be returned from the service.
    type Error;

    /// The type of `HttpService` to be created by this factory.
    type Service: HttpService<
        RequestBody = Self::RequestBody,
        ResponseBody = Self::ResponseBody,
        Error = Self::Error,
    >;

    /// The type of error which will occur during creating an HTTP service.
    type InitError;

    /// A `Future` which will be returned from `new_service` and resolved as a `Service`.
    type Future: Future<Item = Self::Service, Error = Self::InitError>;

    /// Create a new instance of `HttpService` asynchronously.
    fn new_service(&self) -> Self::Future;
}

impl<S: NewHttpService> NewHttpService for Box<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Error = S::Error;
    type Service = S::Service;
    type Future = S::Future;
    type InitError = S::InitError;

    fn new_service(&self) -> Self::Future {
        (**self).new_service()
    }
}

impl<S: NewHttpService> NewHttpService for Arc<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Error = S::Error;
    type Service = S::Service;
    type Future = S::Future;
    type InitError = S::InitError;

    fn new_service(&self) -> Self::Future {
        (**self).new_service()
    }
}

/// Trait representing an asynchronous function from an HTTP request to an HTTP response.
pub trait HttpService {
    /// The type of message body in the request.
    type RequestBody;

    /// The type of message body in the response.
    type ResponseBody;

    /// The type of error which will be returned from this service.
    type Error;

    /// A `Future` which will be returned from `call` and resolved as an HTTP response.
    type Future: Future<Item = Response<Self::ResponseBody>, Error = Self::Error>;

    /// Apply an HTTP request to this service and get a future which will be resolved as an HTTP
    /// response.
    fn call(&mut self, request: Request<Self::RequestBody>) -> Self::Future;
}

impl<S: HttpService> HttpService for Box<S> {
    type RequestBody = S::RequestBody;
    type ResponseBody = S::ResponseBody;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&mut self, request: Request<Self::RequestBody>) -> Self::Future {
        (**self).call(request)
    }
}
