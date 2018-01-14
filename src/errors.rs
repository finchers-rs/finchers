//! Error types thrown from finchers

use std::fmt;
use std::error::Error;
use http::StatusCode;

/// Abstruction of an "error" response.
///
/// This trait is useful for defining the HTTP response of types
/// which implements the [`Error`][error] trait.
/// If the custom error response (like JSON body) is required, use
/// [`Responder`][responder] instead.
///
/// [error]: https://doc.rust-lang.org/stable/std/error/trait.Error.html
/// [responder]: ../trait.Responder.html
pub trait ErrorResponder: Error {
    /// Returns the status code of the HTTP response.
    fn status(&self) -> StatusCode {
        StatusCode::BadRequest
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
    macro_rules! impl_error_responder {
        ($($t:ty,)*) => {
            $(
                impl super::ErrorResponder for $t {}
            )*
        };
    }

    impl_error_responder! {
        ::std::char::ParseCharError,
        ::std::net::AddrParseError,
        ::std::num::ParseIntError,
        ::std::num::ParseFloatError,
        ::std::str::ParseBoolError,
        ::std::string::FromUtf8Error,
        ::std::string::ParseError,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub enum NeverReturn {}

impl fmt::Display for NeverReturn {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unreachable!()
    }
}

impl Error for NeverReturn {
    fn description(&self) -> &str {
        unreachable!()
    }
}

impl ErrorResponder for NeverReturn {}

impl PartialEq for NeverReturn {
    fn eq(&self, _: &Self) -> bool {
        unreachable!()
    }
}

// re-exports
pub use endpoint::body::BodyError;
pub use endpoint::header::EmptyHeader;
pub use endpoint::path::{ExtractPathError, ExtractPathsError};
