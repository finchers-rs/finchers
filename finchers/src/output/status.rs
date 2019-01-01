#![allow(missing_docs)]

use http::{Request, Response, StatusCode};

use super::IntoResponse;
use crate::error::Never;

impl IntoResponse for StatusCode {
    type Body = ();
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = Response::new(());
        *response.status_mut() = self;
        Ok(response)
    }
}

#[derive(Debug)]
pub struct Created<T>(pub T);

impl<T: IntoResponse> IntoResponse for Created<T> {
    type Body = T::Body;
    type Error = T::Error;

    fn into_response(self, request: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = self.0.into_response(request)?;
        *response.status_mut() = StatusCode::CREATED;
        Ok(response)
    }
}

#[derive(Debug)]
pub struct NoContent;

impl IntoResponse for NoContent {
    type Body = ();
    type Error = Never;

    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = Response::new(());
        *response.status_mut() = StatusCode::NO_CONTENT;
        Ok(response)
    }
}

#[derive(Debug)]
pub struct Status<T> {
    pub value: T,
    pub status: StatusCode,
}

impl<T: IntoResponse> IntoResponse for Status<T> {
    type Body = T::Body;
    type Error = T::Error;

    fn into_response(self, request: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = self.value.into_response(request)?;
        *response.status_mut() = self.status;
        Ok(response)
    }
}
