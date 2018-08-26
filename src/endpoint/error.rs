use bitflags::bitflags;
use http::{Method, StatusCode};
use std::fmt;

use crate::error::HttpError;

#[allow(missing_docs)]
pub type EndpointResult<T> = Result<T, EndpointError>;

bitflags! {
    pub(crate) struct Mask: u32 {
        const NOT_MATCHED = 0b_0000_0000_0000;
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

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct EndpointError(Mask);

#[allow(missing_docs)]
impl EndpointError {
    pub(crate) fn not_matched() -> EndpointError {
        EndpointError(Mask::NOT_MATCHED)
    }

    pub(crate) fn method_not_allowed(allowed: &Method) -> Option<EndpointError> {
        macro_rules! pat {
            ($($METHOD:ident),*) => {
                match allowed {
                    $(
                        ref m if *m == Method::$METHOD => Some(EndpointError(Mask::$METHOD)),
                    )*
                    _ => None,
                }
            }
        }
        pat!(GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, TRACE)
    }

    #[inline(always)]
    pub(crate) fn from_mask(mask: Mask) -> EndpointError {
        EndpointError(mask)
    }

    pub(crate) fn merge(self, other: EndpointError) -> EndpointError {
        EndpointError(self.0 | other.0)
    }

    pub fn is_not_matched(self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for EndpointError {
    #[allow(unused_assignments)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_not_matched() {
            f.write_str("no route")
        } else {
            if !f.alternate() {
                return f.write_str("method not allowed");
            }

            write!(f, "method not allowed (allowed methods: ")?;
            for (i, method) in (*self).into_iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?;
                }
                method.fmt(f)?;
            }
            f.write_str(")")
        }
    }
}

impl HttpError for EndpointError {
    fn status_code(&self) -> StatusCode {
        if self.is_not_matched() {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::METHOD_NOT_ALLOWED
        }
    }
}

impl IntoIterator for EndpointError {
    type Item = &'static Method;
    type IntoIter = AllowedMethods;

    fn into_iter(self) -> Self::IntoIter {
        AllowedMethods {
            mask: self.0,
            cursor: Mask::GET,
        }
    }
}

#[derive(Debug)]
pub struct AllowedMethods {
    mask: Mask,
    cursor: Mask,
}

impl Iterator for AllowedMethods {
    type Item = &'static Method;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! dump_method {
            ($m:expr => [$($METHOD:ident),*]) => {$(
                if $m.contains(Mask::$METHOD) { return Some(&Method::$METHOD) }
            )*}
        }
        loop {
            let masked = self.mask & self.cursor;
            self.cursor = Mask::from_bits_truncate(self.cursor.bits() << 1);
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

    #[test]
    fn test_not_matched() {
        let err = EndpointError::not_matched();
        let methods: Vec<_> = err.into_iter().collect();
        assert!(methods.is_empty());
    }

    #[test]
    fn test_method_not_allowed() {
        let err = EndpointError::method_not_allowed(&Method::GET).unwrap();
        let methods: Vec<Method> = err.into_iter().cloned().collect();
        assert_eq!(methods, vec![Method::GET]);
    }

    #[test]
    fn test_merge_1() {
        let err1 = EndpointError::not_matched();
        let err2 = EndpointError::not_matched();
        let err = err1.merge(err2);
        assert!(err.is_not_matched());
    }

    #[test]
    fn test_merge_2() {
        let err1 = EndpointError::not_matched();
        let err2 = EndpointError::method_not_allowed(&Method::GET).unwrap();
        let err = err1.merge(err2);
        assert!(!err.is_not_matched());
    }

    #[test]
    fn test_merge_3() {
        let err1 = EndpointError::method_not_allowed(&Method::GET).unwrap();
        let err2 = EndpointError::method_not_allowed(&Method::POST).unwrap();
        let err = err1.merge(err2);
        assert!(!err.is_not_matched());

        let methods: Vec<Method> = err.into_iter().cloned().collect();
        assert_eq!(methods, vec![Method::GET, Method::POST]);
    }
}
