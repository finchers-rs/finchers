//! Error primitives.

use std::any::TypeId;
use std::error;
use std::fmt;
use std::fmt::{Debug, Display};
use std::io;

use failure;
use failure::Fail;
use http::header::{HeaderMap, HeaderValue};
use http::{header, Response, StatusCode};
use serde::ser::{Serialize, SerializeMap, Serializer};

/// Trait representing error values from endpoints.
///
/// The types which implements this trait will be implicitly converted to an HTTP response
/// by the runtime.
pub trait HttpError: Debug + Display + Send + Sync + 'static {
    /// Return the HTTP status code associated with this error type.
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Append a set of header values to the header map.
    #[allow(unused_variables)]
    fn headers(&self, headers: &mut HeaderMap) {}

    #[allow(missing_docs)]
    fn cause(&self) -> Option<&dyn Fail> {
        None
    }

    #[doc(hidden)]
    fn __private_type_id__(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl HttpError for io::Error {
    fn status_code(&self) -> StatusCode {
        match self.kind() {
            io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
            io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpError for failure::Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.as_fail().cause()
    }
}

/// A type which holds a value of `HttpError` in a type-erased form.
#[derive(Debug)]
pub struct Error(Box<dyn HttpError>);

impl<E: HttpError> From<E> for Error {
    fn from(err: E) -> Self {
        Error(Box::new(err))
    }
}

impl AsRef<dyn HttpError> for Error {
    fn as_ref(&self) -> &dyn HttpError {
        &*self.0
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&*self.0, f)
    }
}

impl Error {
    /// Returns `true` if the type of contained value is the same as `T`.
    pub fn is<T: HttpError>(&self) -> bool {
        self.0.__private_type_id__() == TypeId::of::<T>()
    }

    /// Attempts to downcast the boxed value to a conrete type by reference.
    pub fn downcast_ref<T: HttpError>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(&*self.0 as *const dyn HttpError as *const T)) }
        } else {
            None
        }
    }

    /// Attempts to downcast the boxed value to a conrete type by mutable reference.
    pub fn downcast_mut<T: HttpError>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(&mut *self.0 as *mut dyn HttpError as *mut T)) }
        } else {
            None
        }
    }

    /// Attempts to downcast the boxed value to a conrete type.
    pub fn downcast<T: HttpError>(self) -> Result<T> {
        if self.is::<T>() {
            unsafe {
                Ok(*Box::from_raw(
                    Box::into_raw(self.0) as *mut dyn HttpError as *mut T
                ))
            }
        } else {
            Err(self)
        }
    }

    /// Return the HTTP status code associated with contained value.
    pub fn status_code(&self) -> StatusCode {
        self.0.status_code()
    }

    /// Append a set of header values to the header map.
    pub fn headers(&self, headers: &mut HeaderMap) {
        self.0.headers(headers)
    }

    /// Returns a reference to the underlying cause of contained error value.
    pub fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    pub(crate) fn to_response(&self) -> Response<String> {
        let mut response = Response::new(format!("{:#}", self.0));
        *response.status_mut() = self.status_code();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        self.0.headers(response.headers_mut());
        response
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, ser: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = ser.serialize_map(None)?;
        map.serialize_entry("code", &self.status_code().as_u16())?;
        map.serialize_entry("description", &self.to_string())?;
        // TODO: causes
        map.end()
    }
}

/// A type alias of `Result<T, E>` whose error type is restricted to `Error`.
pub type Result<T> = ::std::result::Result<T, Error>;

// ==== Failure ====

#[derive(Debug)]
struct Failure<F: Fail>(F);

impl<F: Fail> Display for Failure<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<F: Fail> HttpError for Failure<F> {
    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }
}

#[allow(missing_docs)]
pub fn fail(err: impl Fail) -> Error {
    Failure(err).into()
}

// ==== err_msg ====

#[allow(missing_docs)]
pub fn bad_request(msg: impl Debug + Display + Send + Sync + 'static) -> Error {
    err_msg(StatusCode::BAD_REQUEST, msg)
}

#[allow(missing_docs)]
pub fn err_msg(status: StatusCode, msg: impl Debug + Display + Send + Sync + 'static) -> Error {
    ErrorMessage { status, msg }.into()
}

#[derive(Debug)]
struct ErrorMessage<D: fmt::Debug + fmt::Display + Send + 'static> {
    status: StatusCode,
    msg: D,
}

impl<D> fmt::Display for ErrorMessage<D>
where
    D: fmt::Debug + fmt::Display + Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.msg, f)
    }
}

impl<D> HttpError for ErrorMessage<D>
where
    D: fmt::Debug + fmt::Display + Send + Sync + 'static,
{
    fn status_code(&self) -> StatusCode {
        self.status
    }
}

// ==== Never ====

/// A type which has no possible values.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Never {}

impl Never {
    /// Consume itself and transform into an arbitrary type.
    ///
    /// NOTE: This function has never been actually called because the possible values don't exist.
    pub fn never_into<T>(self) -> T {
        match self {}
    }
}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

impl error::Error for Never {
    fn description(&self) -> &str {
        match *self {}
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {}
    }
}

impl HttpError for Never {}
