#![allow(missing_docs)]

use http::{Response, StatusCode};

use super::payload::Empty;
use super::{Output, OutputContext};
use error::Never;

#[derive(Debug)]
pub struct Created<T>(pub T);

impl<T: Output> Output for Created<T> {
    type Body = T::Body;
    type Error = T::Error;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = self.0.respond(cx)?;
        *response.status_mut() = StatusCode::CREATED;
        Ok(response)
    }
}

#[derive(Debug)]
pub struct NoContent;

impl Output for NoContent {
    type Body = Empty;
    type Error = Never;

    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = Response::new(Empty);
        *response.status_mut() = StatusCode::NO_CONTENT;
        Ok(response)
    }
}

#[derive(Debug)]
pub struct Status<T> {
    pub value: T,
    pub status: StatusCode,
}

impl<T: Output> Output for Status<T> {
    type Body = T::Body;
    type Error = T::Error;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = self.value.respond(cx)?;
        *response.status_mut() = self.status;
        Ok(response)
    }
}
