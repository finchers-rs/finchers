//! Components for constructing HTTP responses.

pub mod fs;
pub mod status;

mod binary;
mod debug;
mod json;
mod redirect;
mod text;

use either::Either;
use http::{Request, Response, StatusCode};

pub use self::debug::Debug;
pub use self::fs::NamedFile;
pub use self::json::Json;
pub use self::redirect::Redirect;

/// A trait representing the value to be converted into an HTTP response.
pub trait IntoResponse {
    /// The type of response body.
    type Body;

    /// Converts `self` into an HTTP response.
    fn into_response(self, request: &Request<()>) -> Response<Self::Body>;
}

impl<T> IntoResponse for Response<T> {
    type Body = T;

    #[inline]
    fn into_response(self, _: &Request<()>) -> Response<Self::Body> {
        self
    }
}

impl IntoResponse for () {
    type Body = ();

    fn into_response(self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(());
        *response.status_mut() = StatusCode::NO_CONTENT;
        response
    }
}

impl<T: IntoResponse> IntoResponse for (T,) {
    type Body = T::Body;

    #[inline]
    fn into_response(self, request: &Request<()>) -> Response<Self::Body> {
        self.0.into_response(request)
    }
}

impl<L, R> IntoResponse for Either<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    type Body = izanami_http::Either<L::Body, R::Body>;

    fn into_response(self, request: &Request<()>) -> Response<Self::Body> {
        match self {
            Either::Left(l) => l.into_response(request).map(izanami_http::Either::Left),
            Either::Right(r) => r.into_response(request).map(izanami_http::Either::Right),
        }
    }
}
