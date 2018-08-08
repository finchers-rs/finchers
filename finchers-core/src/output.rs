//! Components for constructing HTTP responses.

pub mod payloads;
pub mod responders;

use http::Response;
use hyper::body::Payload;

use crate::either::Either;
use crate::error::{Error, Never};
use crate::input::Input;

/// Trait representing the conversion to an HTTP response.
#[allow(missing_docs)]
pub trait Responder {
    type Body: Payload;
    type Error: Into<Error>;

    /// Consume `self` and construct a new HTTP response.
    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error>;
}

impl<T: Payload> Responder for Response<T> {
    type Body = T;
    type Error = Never;

    #[inline(always)]
    fn respond(self, _: &Input) -> Result<Response<Self::Body>, Self::Error> {
        Ok(self)
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    Error: From<E>,
{
    type Body = T::Body;
    type Error = Error;

    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error> {
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

    fn respond(self, input: &Input) -> Result<Response<Self::Body>, Self::Error> {
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
