//! Components for constructing HTTP responses.

pub mod body;
pub mod fs;
pub mod status;

mod binary;
mod debug;
mod json;
mod redirect;
mod text;

use either::Either;
use http::{Request, Response, StatusCode};
use std::fmt;

use crate::error::{Error, HttpError, Never};

pub use self::debug::Debug;
pub use self::fs::NamedFile;
pub use self::json::Json;
pub use self::redirect::Redirect;

/// A trait representing the value to be converted into an HTTP response.
pub trait IntoResponse {
    /// The type of response body.
    type Body;

    /// The error type of `respond()`.
    type Error: Into<Error>;

    /// Converts `self` into an HTTP response.
    fn into_response(self, request: &Request<()>) -> Result<Response<Self::Body>, Self::Error>;
}

impl<T> IntoResponse for Response<T> {
    type Body = T;
    type Error = Never;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(self)
    }
}

impl IntoResponse for () {
    type Body = ();
    type Error = Never;

    fn into_response(self, _: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(())
            .unwrap())
    }
}

impl<T: IntoResponse> IntoResponse for (T,) {
    type Body = T::Body;
    type Error = T::Error;

    #[inline]
    fn into_response(self, request: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.into_response(request)
    }
}

impl<T: IntoResponse> IntoResponse for Option<T> {
    type Body = T::Body;
    type Error = Error;

    #[inline]
    fn into_response(self, request: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        self.ok_or_else(|| NoRoute { _priv: () })?
            .into_response(request)
            .map_err(Into::into)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NoRoute {
    _priv: (),
}

impl fmt::Display for NoRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("no route")
    }
}

impl HttpError for NoRoute {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: Into<Error>,
{
    type Body = T::Body;
    type Error = Error;

    #[inline]
    fn into_response(self, request: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        self.map_err(Into::into)?
            .into_response(request)
            .map_err(Into::into)
    }
}

impl<L, R> IntoResponse for Either<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    fn into_response(self, request: &Request<()>) -> Result<Response<Self::Body>, Self::Error> {
        match self {
            Either::Left(l) => l
                .into_response(request)
                .map(|res| res.map(Either::Left))
                .map_err(Into::into),
            Either::Right(r) => r
                .into_response(request)
                .map(|res| res.map(Either::Right))
                .map_err(Into::into),
        }
    }
}
