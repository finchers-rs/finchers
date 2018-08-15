#![allow(missing_docs)]

use http::{Response, StatusCode};
use std::mem::PinMut;

use crate::error::Never;
use crate::input::Input;
use crate::output::payload::Empty;
use crate::output::Responder;

#[derive(Debug)]
pub struct Created<T>(pub T);

impl<T: Responder> Responder for Created<T> {
    type Body = T::Body;
    type Error = T::Error;

    fn respond(self, input: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = self.0.respond(input)?;
        *response.status_mut() = StatusCode::CREATED;
        Ok(response)
    }
}

#[derive(Debug)]
pub struct NoContent;

impl Responder for NoContent {
    type Body = Empty;
    type Error = Never;

    fn respond(self, _: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
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

impl<T: Responder> Responder for Status<T> {
    type Body = T::Body;
    type Error = T::Error;

    fn respond(self, input: PinMut<'_, Input>) -> Result<Response<Self::Body>, Self::Error> {
        let mut response = self.value.respond(input)?;
        *response.status_mut() = self.status;
        Ok(response)
    }
}
