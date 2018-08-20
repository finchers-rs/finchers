use bitflags::bitflags;
use failure::Fail;
use http::header::HeaderMap;
use http::{Method, StatusCode};
use std::fmt;

use self::EndpointErrorKind::*;
use crate::error::{Error, HttpError};

bitflags! {
    pub struct AllowedMethods: u32 {
        const GET     = 0b_0000_0000_0001;
        const POST    = 0b_0000_0000_0010;
        const PUT     = 0b_0000_0000_0100;
        const DELETE  = 0b_0000_0000_1000;
        const HEAD    = 0b_0000_0001_0000;
        const OPTIONS = 0b_0000_0010_0000;
        const CONNECT = 0b_0000_0100_0000;
        const PATCH   = 0b_0000_1000_0000;
        const TRACE   = 0b_0001_0000_0000;
    }
}

impl AllowedMethods {
    pub(crate) fn from_method(method: &Method) -> Option<AllowedMethods> {
        macro_rules! pat {
            ($($METHOD:ident),*) => {
                match method {
                    $(
                        ref m if *m == Method::$METHOD => Some(AllowedMethods::$METHOD),
                    )*
                    _ => None,
                }
            }
        }
        pat!(GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, TRACE)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub enum EndpointErrorKind {
    NotMatched,
    MethodNotAllowed(AllowedMethods),
    Other(Error),
}

impl fmt::Display for EndpointErrorKind {
    #[allow(unused_assignments)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotMatched => f.write_str("no route"),
            MethodNotAllowed(allowed_methods) => {
                if f.alternate() {
                    write!(f, "method not allowed (allowed methods: ")?;

                    macro_rules! dump_method {
                        ($($METHOD:ident),*) => {
                            let mut marked = false;
                            $(
                                if allowed_methods.contains(AllowedMethods::$METHOD) {
                                    if marked {
                                        f.write_str(concat!(", ", stringify!($METHOD)))?;
                                    } else {
                                        f.write_str(stringify!($METHOD))?;
                                    }
                                    marked = true;
                                }
                            )*
                        }
                    }
                    dump_method!(GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, PATCH, TRACE);

                    f.write_str(")")
                } else {
                    f.write_str("method not allowed")
                }
            }
            Other(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

impl HttpError for EndpointErrorKind {
    fn status_code(&self) -> StatusCode {
        match self {
            NotMatched => StatusCode::NOT_FOUND,
            MethodNotAllowed(..) => StatusCode::METHOD_NOT_ALLOWED,
            Other(ref e) => e.status_code(),
        }
    }

    fn headers(&self, h: &mut HeaderMap) {
        match self {
            Other(ref e) => e.headers(h),
            _ => {}
        }
    }

    fn cause(&self) -> Option<&dyn Fail> {
        match self {
            Other(ref e) => e.cause(),
            _ => None,
        }
    }
}

#[allow(missing_docs)]
pub type EndpointResult<F> = Result<F, EndpointErrorKind>;
