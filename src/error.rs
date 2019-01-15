//! Error primitives.

use {
    crate::{output::IntoResponse, util::Never},
    failure::{AsFail, Fail},
    http::{Request, Response, StatusCode},
    std::{any::TypeId, fmt, io},
};

/// Trait that abstracts the error representation used in Finchers.
///
/// Roughly speaking, this trait adds some context around HTTP to
/// `failure::Fail`.
pub trait HttpError: AsFail + Send + Sync + 'static {
    /// Returns an HTTP status code associated with this error value.
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Creates an HTTP response without the request body.
    fn to_response(&self, _: &Request<()>) -> Response<()> {
        let mut response = Response::new(());
        *response.status_mut() = self.status_code();
        response
    }

    // not a public API.
    #[doc(hidden)]
    fn __private_type_id__(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl dyn HttpError {
    /// Returns `true` if the type of contained value is the same as `T`.
    pub fn is<T: HttpError>(&self) -> bool {
        self.__private_type_id__() == TypeId::of::<T>()
    }

    /// Attempts to downcast the boxed value to a conrete type by reference.
    pub fn downcast_ref<T: HttpError>(&self) -> Option<&T> {
        if self.is::<T>() {
            Some(unsafe { &*(&*self as *const dyn HttpError as *const T) })
        } else {
            None
        }
    }

    /// Attempts to downcast the boxed value to a conrete type by mutable reference.
    pub fn downcast_mut<T: HttpError>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            Some(unsafe { &mut *(&mut *self as *mut dyn HttpError as *mut T) })
        } else {
            None
        }
    }

    /// Attempts to downcast the boxed value to a conrete type.
    pub fn downcast<T: HttpError>(self: Box<Self>) -> std::result::Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            Ok(unsafe { Box::from_raw(Box::into_raw(self) as *mut T) })
        } else {
            Err(self)
        }
    }
}

impl fmt::Debug for dyn HttpError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_fail(), f)
    }
}

impl fmt::Display for dyn HttpError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_fail(), f)
    }
}

impl Fail for dyn HttpError {
    #[inline]
    fn name(&self) -> Option<&str> {
        self.as_fail().name()
    }

    #[inline]
    fn cause(&self) -> Option<&dyn Fail> {
        self.as_fail().cause()
    }

    // TODO: backtrace
}

impl HttpError for Never {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }

    fn to_response(&self, _: &Request<()>) -> Response<()> {
        match *self {}
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
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<E> HttpError for failure::SyncFailure<E>
where
    E: std::error::Error + Send + 'static,
{
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

// ==== ErrorMessage ====

/// Creates a value of`Error` from the specific message and status code.
pub fn err_msg<D>(msg: D, status: StatusCode) -> Error
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    #[derive(Debug, failure::Fail)]
    #[fail(display = "{}", msg)]
    struct ErrorMessage<D>
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        msg: D,
        status: StatusCode,
    }

    impl<D> HttpError for ErrorMessage<D>
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        fn status_code(&self) -> StatusCode {
            self.status
        }
    }

    ErrorMessage { msg, status }.into()
}

macro_rules! define_error_fn {
    ($(
        $name:ident => $STATUS:ident,
    )*) => {$(
        #[allow(missing_docs)]
        pub fn $name<D>(msg: D) -> Error
        where
            D: fmt::Display + fmt::Debug + Send + Sync + 'static,
        {
            err_msg(msg, StatusCode::$STATUS)
        }
    )*};
}

define_error_fn! {
    bad_request => BAD_REQUEST,
    unauthorized => UNAUTHORIZED,
    forbidden => FORBIDDEN,
    not_found => NOT_FOUND,
    method_not_allowed => METHOD_NOT_ALLOWED,
    internal_server_error => INTERNAL_SERVER_ERROR,
}

/// Creates a value of `Error` from the specific error value.
pub fn fail<E>(error: E, status: StatusCode) -> Error
where
    E: AsFail + Send + Sync + 'static,
{
    #[derive(Debug)]
    struct Failure<E>
    where
        E: AsFail + Send + Sync + 'static,
    {
        error: E,
        status: StatusCode,
    }

    impl<E> fmt::Display for Failure<E>
    where
        E: AsFail + Send + Sync + 'static,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(self.as_fail(), f)
        }
    }

    impl<E> AsFail for Failure<E>
    where
        E: AsFail + Send + Sync + 'static,
    {
        fn as_fail(&self) -> &dyn Fail {
            self.error.as_fail()
        }
    }

    impl<E> HttpError for Failure<E>
    where
        E: AsFail + Send + Sync + 'static,
    {
        fn status_code(&self) -> StatusCode {
            self.status
        }
    }

    Failure { error, status }.into()
}

// ==== Error ====

/// A type which holds a value of `HttpError` in a type-erased form.
#[derive(Debug)]
pub struct Error {
    inner: Box<dyn HttpError>,
}

impl AsRef<dyn HttpError> for Error {
    fn as_ref(&self) -> &dyn HttpError {
        &*self.inner
    }
}

impl AsMut<dyn HttpError> for Error {
    fn as_mut(&mut self) -> &mut dyn HttpError {
        &mut *self.inner
    }
}

impl std::ops::Deref for Error {
    type Target = dyn HttpError;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl std::ops::DerefMut for Error {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<E: HttpError> From<E> for Error {
    fn from(err: E) -> Self {
        Self::new(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.inner, formatter)
    }
}

impl IntoResponse for Error {
    type Body = String;

    fn into_response(self, request: &Request<()>) -> Response<Self::Body> {
        self.into_response_with(request, |err, _, _| err.to_string())
    }
}

impl Error {
    #[allow(missing_docs)]
    pub fn new<E>(err: E) -> Self
    where
        E: HttpError,
    {
        Self {
            inner: Box::new(err),
        }
    }

    /// Attempts to downcast the boxed value to a conrete type.
    pub fn downcast<T: HttpError>(self) -> Result<T> {
        self.inner
            .downcast::<T>()
            .map(|e| *e)
            .map_err(|inner| Self { inner })
    }

    #[allow(missing_docs)]
    pub fn into_response_with<F, T>(self, request: &Request<()>, f: F) -> Response<T>
    where
        F: Fn(&Self, &Request<()>, &mut Response<()>) -> T,
    {
        let mut response = self.inner.to_response(request);
        let body = f(&self, request, &mut response);
        response.extensions_mut().insert(self);
        response.map(|_| body)
    }
}

/// A type alias of `Result<T, E>` whose error type is restricted to `Error`.
pub type Result<T> = std::result::Result<T, Error>;
