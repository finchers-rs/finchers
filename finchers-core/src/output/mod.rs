//! Components for constructing HTTP responses.

pub mod payloads;
pub mod responders;

use std::mem::PinMut;

use http::{Response, StatusCode};
use hyper::body::Payload;

use self::payloads::Empty;
use either::Either;
use error::{Error, Never, NoRoute};
use input::Input;

/// Trait representing types to be converted into an HTTP response.
pub trait Responder {
    /// The type of message body in the HTTP response to the client.
    type Body: Payload;

    /// The error type which will be returned from `respond()`.
    type Error: Into<Error>;

    /// Performs conversion this value into an HTTP response.
    fn respond(self, input: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error>;
}

impl<T: Payload> Responder for Response<T> {
    type Body = T;
    type Error = Never;

    #[inline(always)]
    fn respond(self, _: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(self)
    }
}

impl Responder for () {
    type Body = Empty;
    type Error = Never;

    fn respond(self, _: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Empty)
            .unwrap())
    }
}

impl<T: Responder> Responder for (T,) {
    type Body = T::Body;
    type Error = T::Error;

    #[inline]
    fn respond(self, input: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.respond(input)
    }
}

impl<T> Responder for Option<T>
where
    T: Responder,
{
    type Body = T::Body;
    type Error = Error;

    fn respond(self, input: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
        self.ok_or_else(|| NoRoute)?
            .respond(input)
            .map_err(Into::into)
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    Error: From<E>,
{
    type Body = T::Body;
    type Error = Error;

    fn respond(self, input: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
        self?.respond(input).map_err(Into::into)
    }
}

impl<L, R> Responder for Either<L, R>
where
    L: Responder,
    R: Responder,
{
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    fn respond(self, input: PinMut<Input>) -> Result<Response<Self::Body>, Self::Error> {
        match self {
            Either::Left(l) => l
                .respond(input)
                .map(|res| res.map(Either::Left))
                .map_err(Into::into),
            Either::Right(r) => r
                .respond(input)
                .map(|res| res.map(Either::Right))
                .map_err(Into::into),
        }
    }
}
