//! Components for constructing HTTP responses.

pub mod body;
pub mod fs;
pub mod status;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.7",
    note = "use `output::body` instead."
)]
pub mod payload {
    pub use super::body::*;
}

mod binary;
mod debug;
mod json;
mod text;

use either::Either;
use http::{Response, StatusCode};
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use error::{Error, HttpError, Never};
use input::Input;

use self::body::ResBody;

#[doc(hidden)]
#[allow(deprecated)]
pub use self::binary::Binary;
pub use self::debug::Debug;
pub use self::fs::NamedFile;
pub use self::json::Json;
#[doc(hidden)]
#[allow(deprecated)]
pub use self::text::Text;

/// Contextual information at applying `Output::respond`.
#[derive(Debug)]
pub struct OutputContext<'a> {
    input: &'a mut Input,
    pretty: bool,
    _marker: PhantomData<Rc<()>>,
}

impl<'a> OutputContext<'a> {
    pub(crate) fn new(input: &'a mut Input) -> OutputContext<'a> {
        OutputContext {
            input,
            pretty: false,
            _marker: PhantomData,
        }
    }

    /// Returns a pinned reference to `Input` stored on the task context.
    pub fn input(&mut self) -> &mut Input {
        &mut *self.input
    }

    /// Creates a clone of `OutputContext` with setting the mode to "pretty".
    pub fn pretty(&mut self) -> OutputContext<'_> {
        OutputContext {
            input: self.input(),
            pretty: true,
            _marker: PhantomData,
        }
    }

    /// Returns `true` if the current mode is set to "pretty".
    pub fn is_pretty(&self) -> bool {
        self.pretty
    }
}

/// A trait representing the value to be converted into an HTTP response.
pub trait Output: Sized {
    /// The type of response body.
    type Body: ResBody;

    /// The error type of `respond()`.
    type Error: Into<Error>;

    /// Converts `self` into an HTTP response.
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error>;
}

impl<T: ResBody> Output for Response<T> {
    type Body = T;
    type Error = Never;

    #[inline(always)]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(self)
    }
}

impl Output for () {
    type Body = ();
    type Error = Never;

    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(())
            .unwrap())
    }
}

impl<T: Output> Output for (T,) {
    type Body = T::Body;
    type Error = T::Error;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.respond(cx)
    }
}

impl<T: Output> Output for Option<T> {
    type Body = T::Body;
    type Error = Error;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        self.ok_or_else(|| NoRoute { _priv: () })?
            .respond(cx)
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

impl<T, E> Output for Result<T, E>
where
    T: Output,
    E: Into<Error>,
{
    type Body = T::Body;
    type Error = Error;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        self.map_err(Into::into)?.respond(cx).map_err(Into::into)
    }
}

impl<L, R> Output for Either<L, R>
where
    L: Output,
    R: Output,
{
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        match self {
            Either::Left(l) => l
                .respond(cx)
                .map(|res| res.map(Either::Left))
                .map_err(Into::into),
            Either::Right(r) => r
                .respond(cx)
                .map(|res| res.map(Either::Right))
                .map_err(Into::into),
        }
    }
}

/// A wrapper type to use pretty output for the internal value.
#[derive(Debug)]
pub struct Pretty<T>(pub T);

impl<T: Output> Output for Pretty<T> {
    type Body = T::Body;
    type Error = T::Error;

    #[inline]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.respond(&mut cx.pretty())
    }
}
