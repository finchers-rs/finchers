//! Definition of `EndpointError` and supplemental components.

use bitflags::bitflags;
use http::{Method, StatusCode};
use std::fmt;
use std::ops::{BitOr, BitOrAssign};

use crate::error::HttpError;

/// A type alias of `Result<T, E>` with the error type fixed at `EndpointError`.
pub type EndpointResult<T> = Result<T, EndpointError>;

/// A type representing error values returned from `Endpoint::apply()`.
///
/// This error type represents the errors around routing determined
/// before executing the `Future` returned from the endpoint.
#[derive(Debug)]
pub struct EndpointError(EndpointErrorKind);

#[derive(Debug, Copy, Clone)]
enum EndpointErrorKind {
    NotMatched,
    MethodNotAllowed(AllowedMethods),
    InvalidRequest(InvalidRequest),
}

#[derive(Debug, Copy, Clone)]
enum InvalidRequest {
    MissingHeader(&'static str),
    MissingQuery,
}

impl fmt::Display for InvalidRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidRequest::MissingHeader(name) => write!(f, "missing header: `{}'", name),
            InvalidRequest::MissingQuery => f.write_str("missing query"),
        }
    }
}

impl EndpointError {
    /// Create a value of `EndpointError` with an annotation that
    /// the current endpoint does not match to the provided request.
    pub fn not_matched() -> EndpointError {
        EndpointError(EndpointErrorKind::NotMatched)
    }

    /// Create a value of `EndpointError` whth an annotation that
    /// the current endpoint does not matche to the provided HTTP method.
    pub fn method_not_allowed(allowed: AllowedMethods) -> EndpointError {
        EndpointError(EndpointErrorKind::MethodNotAllowed(allowed))
    }

    pub(crate) fn missing_header(name: &'static str) -> EndpointError {
        EndpointError(EndpointErrorKind::InvalidRequest(
            InvalidRequest::MissingHeader(name),
        ))
    }

    pub(crate) fn missing_query() -> EndpointError {
        EndpointError(EndpointErrorKind::InvalidRequest(
            InvalidRequest::MissingQuery,
        ))
    }

    #[doc(hidden)]
    pub fn merge(self, other: EndpointError) -> EndpointError {
        use self::EndpointErrorKind::*;
        EndpointError(match (self.0, other.0) {
            (NotMatched, NotMatched) => NotMatched,
            (NotMatched, MethodNotAllowed(allowed)) => MethodNotAllowed(allowed),
            (NotMatched, InvalidRequest(reason)) => InvalidRequest(reason),
            (MethodNotAllowed(allowed), NotMatched) => MethodNotAllowed(allowed),
            (MethodNotAllowed(allowed1), MethodNotAllowed(allowed2)) => {
                MethodNotAllowed(AllowedMethods(allowed1.0 | allowed2.0))
            }
            (MethodNotAllowed(..), InvalidRequest(reason)) => InvalidRequest(reason),
            (InvalidRequest(reason), ..) => InvalidRequest(reason),
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
            InvalidRequest(ref reason) => fmt::Display::fmt(reason, f),
        }
    }
}

impl HttpError for EndpointError {
    fn status_code(&self) -> StatusCode {
        match self.0 {
            EndpointErrorKind::NotMatched => StatusCode::NOT_FOUND,
            EndpointErrorKind::MethodNotAllowed(..) => StatusCode::METHOD_NOT_ALLOWED,
            EndpointErrorKind::InvalidRequest(..) => StatusCode::BAD_REQUEST,
        }
    }
}

bitflags! {
    pub(crate) struct AllowedMethodsMask: u32 {
        const GET         = 0b_0000_0000_0001;
        const POST        = 0b_0000_0000_0010;
        const PUT         = 0b_0000_0000_0100;
        const DELETE      = 0b_0000_0000_1000;
        const HEAD        = 0b_0000_0001_0000;
        const OPTIONS     = 0b_0000_0010_0000;
        const CONNECT     = 0b_0000_0100_0000;
        const PATCH       = 0b_0000_1000_0000;
        const TRACE       = 0b_0001_0000_0000;
    }
}

/// A collection type which represents a set of allowed HTTP methods.
#[derive(Debug, Clone, Copy)]
pub struct AllowedMethods(pub(crate) AllowedMethodsMask);

macro_rules! define_allowed_methods_constructors {
    ($($METHOD:ident,)*) => {$(
        #[allow(missing_docs)]
        pub const $METHOD: AllowedMethods = AllowedMethods(AllowedMethodsMask::$METHOD);
    )*};
}

impl AllowedMethods {
    define_allowed_methods_constructors![
        GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, TRACE,
    ];

    #[allow(missing_docs)]
    pub fn from_http(allowed: &Method) -> Option<AllowedMethods> {
        macro_rules! pat {
            ($($METHOD:ident),*) => {
                match allowed {
                    $(
                        ref m if *m == Method::$METHOD => Some(AllowedMethods(AllowedMethodsMask::$METHOD)),
                    )*
                    _ => None,
                }
            }
        }
        pat!(GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, TRACE)
    }
}

impl BitOr for AllowedMethods {
    type Output = AllowedMethods;

    #[inline]
    fn bitor(self, other: AllowedMethods) -> Self::Output {
        AllowedMethods(self.0 | other.0)
    }
}

impl BitOrAssign for AllowedMethods {
    #[inline]
    fn bitor_assign(&mut self, other: AllowedMethods) {
        self.0 |= other.0;
    }
}

impl IntoIterator for AllowedMethods {
    type Item = &'static Method;
    type IntoIter = AllowedMethodsIter;

    fn into_iter(self) -> Self::IntoIter {
        AllowedMethodsIter {
            allowed: self.0,
            cursor: AllowedMethodsMask::GET,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AllowedMethodsIter {
    allowed: AllowedMethodsMask,
    cursor: AllowedMethodsMask,
}

impl Iterator for AllowedMethodsIter {
    type Item = &'static Method;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! dump_method {
            ($m:expr => [$($METHOD:ident),*]) => {$(
                if $m.contains(AllowedMethodsMask::$METHOD) { return Some(&Method::$METHOD) }
            )*}
        }
        loop {
            let masked = self.allowed & self.cursor;
            self.cursor = AllowedMethodsMask::from_bits_truncate(self.cursor.bits() << 1);
            if self.cursor.is_empty() {
                return None;
            }
            dump_method!(masked => [
                GET,
                POST,
                PUT,
                DELETE,
                HEAD,
                OPTIONS,
                CONNECT,
                PATCH,
                TRACE
            ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::assert_matches;

    #[test]
    fn test_methods_single_get() {
        let methods: Vec<Method> = AllowedMethods(AllowedMethodsMask::GET)
            .into_iter()
            .cloned()
            .collect();
        assert_eq!(methods, vec![Method::GET]);
    }

    #[test]
    fn test_methods_two_methods() {
        let methods: Vec<Method> =
            AllowedMethods(AllowedMethodsMask::GET | AllowedMethodsMask::POST)
                .into_iter()
                .cloned()
                .collect();
        assert_eq!(methods, vec![Method::GET, Method::POST]);
    }

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
        let err2 = EndpointError::method_not_allowed(AllowedMethods(AllowedMethodsMask::GET));
        assert_matches!(
            err1.merge(err2).0,
            EndpointErrorKind::MethodNotAllowed(allowed) if allowed.0 == AllowedMethodsMask::GET
        );
    }

    #[test]
    fn test_merge_3() {
        let err1 = EndpointError::method_not_allowed(AllowedMethods(AllowedMethodsMask::GET));
        let err2 = EndpointError::method_not_allowed(AllowedMethods(AllowedMethodsMask::POST));
        assert_matches!(
            err1.merge(err2).0,
            EndpointErrorKind::MethodNotAllowed(allowed) if allowed.0 == AllowedMethodsMask::GET | AllowedMethodsMask::POST
        );
    }
}
