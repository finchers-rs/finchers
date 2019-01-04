//! Definition of `ApplyError` and supplemental components.

use http::StatusCode;
use std::fmt;

use crate::endpoint::syntax::verb::Verbs;
use crate::error::Error;

/// A type alias of `Result<T, E>` with the error type fixed at `ApplyError`.
pub type ApplyResult<T> = Result<T, ApplyError>;

/// A type representing error values returned from `Endpoint::apply()`.
///
/// This error type represents the errors around routing determined
/// before executing the `Future` returned from the endpoint.
#[derive(Debug)]
pub struct ApplyError(ApplyErrorKind);

#[derive(Debug)]
enum ApplyErrorKind {
    NotMatched,
    MethodNotAllowed(Verbs),
    Custom(Error),
}

impl ApplyError {
    /// Create a value of `ApplyError` with an annotation that
    /// the current endpoint does not match to the provided request.
    pub fn not_matched() -> ApplyError {
        ApplyError(ApplyErrorKind::NotMatched)
    }

    /// Create a value of `ApplyError` with an annotation that
    /// the current endpoint does not matche to the provided HTTP method.
    pub fn method_not_allowed(allowed: Verbs) -> ApplyError {
        ApplyError(ApplyErrorKind::MethodNotAllowed(allowed))
    }

    /// Create a value of `ApplyError` from the custom error value.
    ///
    /// The generated error has the HTTP status code `400 Bad Request`.
    pub fn custom(err: impl Into<Error>) -> ApplyError {
        ApplyError(ApplyErrorKind::Custom(err.into()))
    }

    #[doc(hidden)]
    pub fn merge(self, other: ApplyError) -> ApplyError {
        use self::ApplyErrorKind::*;
        ApplyError(match (self.0, other.0) {
            (Custom(reason), _) => Custom(reason),
            (_, Custom(reason)) => Custom(reason),

            (MethodNotAllowed(allowed1), MethodNotAllowed(allowed2)) => {
                MethodNotAllowed(allowed1 | allowed2)
            }
            (NotMatched, MethodNotAllowed(allowed)) => MethodNotAllowed(allowed),
            (MethodNotAllowed(allowed), NotMatched) => MethodNotAllowed(allowed),

            (NotMatched, NotMatched) => NotMatched,
        })
    }
}

impl fmt::Display for ApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ApplyErrorKind::*;
        match self.0 {
            NotMatched => formatter.write_str("not matched"),
            MethodNotAllowed(..) if !formatter.alternate() => {
                formatter.write_str("method not allowed")
            }
            MethodNotAllowed(allowed) => {
                write!(formatter, "method not allowed (allowed methods: ")?;
                for (i, method) in allowed.into_iter().enumerate() {
                    if i > 0 {
                        formatter.write_str(", ")?;
                    }
                    method.fmt(formatter)?;
                }
                formatter.write_str(")")
            }
            Custom(ref err) => fmt::Display::fmt(err, formatter),
        }
    }
}

impl Into<Error> for ApplyError {
    fn into(self) -> Error {
        use self::ApplyErrorKind::*;
        match self.0 {
            NotMatched => StatusCode::NOT_FOUND.into(),
            MethodNotAllowed(..) => StatusCode::METHOD_NOT_ALLOWED.into(),
            Custom(err) => err,
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
        let err1 = ApplyError::not_matched();
        let err2 = ApplyError::not_matched();
        let err = err1.merge(err2);
        assert_matches!(err.0, ApplyErrorKind::NotMatched);
    }

    #[test]
    fn test_merge_2() {
        let err1 = ApplyError::not_matched();
        let err2 = ApplyError::method_not_allowed(Verbs::GET);
        assert_matches!(
            err1.merge(err2).0,
            ApplyErrorKind::MethodNotAllowed(allowed) if allowed.contains(&Method::GET)
        );
    }

    #[test]
    fn test_merge_3() {
        let err1 = ApplyError::method_not_allowed(Verbs::GET);
        let err2 = ApplyError::method_not_allowed(Verbs::POST);
        assert_matches!(
            err1.merge(err2).0,
            ApplyErrorKind::MethodNotAllowed(allowed) if allowed.contains(&Method::GET) && allowed.contains(&Method::POST)
        );
    }
}
