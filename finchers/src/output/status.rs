#![allow(missing_docs)]

use http::{Request, Response, StatusCode};

use super::IntoResponse;

impl IntoResponse for StatusCode {
    type Body = &'static [u8];

    #[inline]
    fn into_response(self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(&[] as &[u8]);
        *response.status_mut() = self;
        response
    }
}

#[derive(Debug)]
pub struct Created<T>(pub T);

impl<T: IntoResponse> IntoResponse for Created<T> {
    type Body = T::Body;

    fn into_response(self, request: &Request<()>) -> Response<Self::Body> {
        let mut response = self.0.into_response(request);
        *response.status_mut() = StatusCode::CREATED;
        response
    }
}

#[derive(Debug)]
pub struct NoContent;

impl IntoResponse for NoContent {
    type Body = ();

    fn into_response(self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(());
        *response.status_mut() = StatusCode::NO_CONTENT;
        response
    }
}

#[derive(Debug)]
pub struct Status<T> {
    pub value: T,
    pub status: StatusCode,
}

impl<T: IntoResponse> IntoResponse for Status<T> {
    type Body = T::Body;

    fn into_response(self, request: &Request<()>) -> Response<Self::Body> {
        let mut response = self.value.into_response(request);
        *response.status_mut() = self.status;
        response
    }
}
