use futures::Future;
use http::{Request, Response};
use std::sync::Arc;

pub trait NewHttpService {
    type RequestBody;
    type ResponseBody;
    type Error;
    type Service: HttpService<RequestBody = Self::RequestBody, ResponseBody = Self::ResponseBody, Error = Self::Error>;
    type InitError;
    type Future: Future<Item = Self::Service, Error = Self::InitError>;

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

#[allow(missing_docs)]
pub trait HttpService {
    type RequestBody;
    type ResponseBody;
    type Error;
    type Future: Future<Item = Response<Self::ResponseBody>, Error = Self::Error>;

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
