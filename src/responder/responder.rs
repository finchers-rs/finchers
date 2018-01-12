use std::borrow::Cow;
use std::error::Error;
use http::{Headers, IntoBody, Response, StatusCode};

/// Abstrcution of types converted into a raw HTTP response.
pub trait Responder {
    /// The type of the value returned from `body`
    type Body: IntoBody;

    /// Returns the status code of the HTTP response
    ///
    /// The default value is `200 OK`.
    fn status(&self) -> StatusCode {
        StatusCode::Ok
    }

    /// Returns the instance of response body, if available.
    ///
    /// The default value is `None`.
    fn body(&mut self) -> Option<Self::Body> {
        None
    }

    /// Add additional headers to the response.
    ///
    /// By default, this method has no affect to the HTTP response.
    fn headers(&self, &mut Headers) {}

    #[allow(missing_docs)]
    fn respond(&mut self) -> Response {
        super::respond(self)
    }
}

impl Responder for () {
    type Body = ();

    fn status(&self) -> StatusCode {
        StatusCode::NoContent
    }
}

/// Abstrcution of types to be convert to a `Responder`.
pub trait IntoResponder {
    /// The type of returned value from `into_response`
    type Responder: Responder;

    /// Convert itself into `Self::Responder`
    fn into_responder(self) -> Self::Responder;
}

impl<R: Responder> IntoResponder for R {
    type Responder = Self;

    fn into_responder(self) -> Self {
        self
    }
}

/// A responder with the body of string.
#[derive(Debug)]
pub struct StringResponder(Option<Cow<'static, str>>);

impl Responder for StringResponder {
    type Body = Cow<'static, str>;

    fn body(&mut self) -> Option<Self::Body> {
        self.0.take()
    }
}

impl IntoResponder for &'static str {
    type Responder = StringResponder;

    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self.into()))
    }
}

impl IntoResponder for String {
    type Responder = StringResponder;

    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self.into()))
    }
}

impl IntoResponder for Cow<'static, str> {
    type Responder = StringResponder;

    fn into_responder(self) -> Self::Responder {
        StringResponder(Some(self))
    }
}

/// Abstruction of an "error" response.
///
/// This trait is useful for defining the HTTP response of types
/// which implements the [`Error`][error] trait.
/// If the own error response (like JSON body) is required, use
/// `Responder` directly.
///
/// [error]: https://doc.rust-lang.org/stable/std/error/trait.Error.html
pub trait ErrorResponder: Error {
    /// Returns the status code of the HTTP response.
    fn status(&self) -> StatusCode {
        StatusCode::InternalServerError
    }

    /// Returns the message string of the HTTP response.
    fn message(&self) -> Option<String> {
        Some(format!(
            "description: {}\ndetail: {}",
            Error::description(self),
            self
        ))
    }
}

mod implementors {
    use super::*;
    use std::string::{FromUtf8Error, ParseError};
    use http::HttpError;

    impl ErrorResponder for FromUtf8Error {
        fn status(&self) -> StatusCode {
            StatusCode::BadRequest
        }
    }

    impl ErrorResponder for ParseError {
        fn status(&self) -> StatusCode {
            StatusCode::BadRequest
        }
    }

    impl ErrorResponder for HttpError {}
}

impl<E: ErrorResponder> Responder for E {
    type Body = String;

    fn status(&self) -> StatusCode {
        ErrorResponder::status(self)
    }

    fn body(&mut self) -> Option<Self::Body> {
        self.message()
    }
}
