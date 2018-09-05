//! Definition of `EndpointError` and supplemental components.

use failure::{Error, Fail};
use http::StatusCode;
use std::fmt;

use crate::endpoint::syntax::verb::Verbs;
use crate::error::HttpError;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.4",
    note = "renamed to `endpoint::syntax::verb::Verbs`."
)]
pub use crate::endpoint::syntax::verb::Verbs as AllowedMethods;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.4",
    note = "renamed to `endpoint::syntax::verb::VerbsIter`."
)]
pub use crate::endpoint::syntax::verb::VerbsIter as AllowedMethodsIter;

/// A type alias of `Result<T, E>` with the error type fixed at `EndpointError`.
pub type EndpointResult<T> = Result<T, EndpointError>;

/// A type representing error values returned from `Endpoint::apply()`.
///
/// This error type represents the errors around routing determined
/// before executing the `Future` returned from the endpoint.
#[derive(Debug)]
pub struct EndpointError(EndpointErrorKind);

#[derive(Debug)]
enum EndpointErrorKind {
    NotMatched,
    MethodNotAllowed(Verbs),
    Other(Error),
}

impl EndpointError {
    /// Create a value of `EndpointError` with an annotation that
    /// the current endpoint does not match to the provided request.
    pub fn not_matched() -> EndpointError {
        EndpointError(EndpointErrorKind::NotMatched)
    }

    /// Create a value of `EndpointError` with an annotation that
    /// the current endpoint does not matche to the provided HTTP method.
    pub fn method_not_allowed(allowed: Verbs) -> EndpointError {
        EndpointError(EndpointErrorKind::MethodNotAllowed(allowed))
    }

    /// Create a value of `EndpointError` from the custom error value.
    ///
    /// The generated error has the HTTP status code `400 Bad Request`.
    pub fn custom(cause: impl Into<Error>) -> EndpointError {
        EndpointError(EndpointErrorKind::Other(cause.into()))
    }

    #[doc(hidden)]
    pub fn merge(self, other: EndpointError) -> EndpointError {
        use self::EndpointErrorKind::*;
        EndpointError(match (self.0, other.0) {
            (Other(reason), _) => Other(reason),
            (_, Other(reason)) => Other(reason),

            (MethodNotAllowed(allowed1), MethodNotAllowed(allowed2)) => {
                MethodNotAllowed(allowed1 | allowed2)
            }
            (NotMatched, MethodNotAllowed(allowed)) => MethodNotAllowed(allowed),
            (MethodNotAllowed(allowed), NotMatched) => MethodNotAllowed(allowed),

            (NotMatched, NotMatched) => NotMatched,
        })
    }
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::EndpointErrorKind::*;
        match self.0 {
            NotMatched => f.write_str("not matched"),
            MethodNotAllowed(..) if !f.alternate() => f.write_str("method not allowed"),
            MethodNotAllowed(allowed) => {
                write!(f, "method not allowed (allowed methods: ")?;
                for (i, method) in allowed.into_iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    method.fmt(f)?;
                }
                f.write_str(")")
            }
            Other(ref reason) => fmt::Display::fmt(reason, f),
        }
    }
}

impl HttpError for EndpointError {
    fn status_code(&self) -> StatusCode {
        match self.0 {
            EndpointErrorKind::NotMatched => StatusCode::NOT_FOUND,
            EndpointErrorKind::MethodNotAllowed(..) => StatusCode::METHOD_NOT_ALLOWED,
            EndpointErrorKind::Other(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn cause(&self) -> Option<&dyn Fail> {
        match self.0 {
            EndpointErrorKind::Other(ref cause) => Some(cause.as_fail()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Method;
    use matches::assert_matches;

    #[test]
    fn test_merge_1() {
        let err1 = EndpointError::not_matched();
        let err2 = EndpointError::not_matched();
        let err = err1.merge(err2);
        assert_matches!(err.0, EndpointErrorKind::NotMatched);
    }

    #[test]
    fn test_merge_2() {
        let err1 = EndpointError::not_matched();
        let err2 = EndpointError::method_not_allowed(Verbs::GET);
        assert_matches!(
            err1.merge(err2).0,
            EndpointErrorKind::MethodNotAllowed(allowed) if allowed.contains(&Method::GET)
        );
    }

    #[test]
    fn test_merge_3() {
        let err1 = EndpointError::method_not_allowed(Verbs::GET);
        let err2 = EndpointError::method_not_allowed(Verbs::POST);
        assert_matches!(
            err1.merge(err2).0,
            EndpointErrorKind::MethodNotAllowed(allowed) if allowed.contains(&Method::GET) && allowed.contains(&Method::POST)
        );
    }
}
