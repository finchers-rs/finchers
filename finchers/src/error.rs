//! Error primitives.

use {
    crate::{output::IntoResponse, util::Never},
    bytes::Bytes,
    http::{Request, Response, StatusCode},
    std::{any::Any, fmt, io},
};

/// Trait representing error values from endpoints.
///
/// The types which implements this trait will be implicitly converted to an HTTP response
/// by the runtime.
pub trait HttpError: fmt::Debug + fmt::Display + Send + Sync + 'static {
    type Body: Into<Bytes>;

    fn status_code(&self) -> StatusCode;

    fn to_response(&self, _: &Request<()>) -> Response<Self::Body>;
}

impl HttpError for Never {
    type Body = Bytes;

    fn status_code(&self) -> StatusCode {
        match *self {}
    }

    fn to_response(&self, _: &Request<()>) -> Response<Self::Body> {
        match *self {}
    }
}

impl HttpError for StatusCode {
    type Body = Bytes;

    fn status_code(&self) -> StatusCode {
        *self
    }

    fn to_response(&self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(Bytes::new());
        *response.status_mut() = *self;
        response
    }
}

impl HttpError for io::Error {
    type Body = String;

    fn status_code(&self) -> StatusCode {
        match self.kind() {
            io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
            io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn to_response(&self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(format!("I/O error: {}", self));
        *response.status_mut() = self.status_code();
        response
    }
}

impl HttpError for failure::Error {
    type Body = String;

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn to_response(&self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(self.to_string());
        *response.status_mut() = self.status_code();
        response
    }
}

impl<E> HttpError for failure::SyncFailure<E>
where
    E: std::error::Error + Send + 'static,
{
    type Body = String;

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn to_response(&self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(self.to_string());
        *response.status_mut() = self.status_code();
        response
    }
}

/// A wrapper for providing an implementation of `HttpError` with the status code `400 Bad Request`.
#[derive(Debug)]
pub struct BadRequest<E>(E);

impl<E> BadRequest<E>
where
    E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    #[inline]
    pub fn into_inner(self) -> E {
        self.0
    }
}

impl<E> From<E> for BadRequest<E>
where
    E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        BadRequest(err)
    }
}

impl<E> fmt::Display for BadRequest<E>
where
    E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<E> HttpError for BadRequest<E>
where
    E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    type Body = String;

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn to_response(&self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(self.to_string());
        *response.status_mut() = self.status_code();
        response
    }
}

/// A wrapper for providing an implementation of `HttpError` for `Display`able types.
#[derive(Debug)]
pub struct InternalServerError<E>(E);

impl<E> InternalServerError<E>
where
    E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    #[inline]
    pub fn into_inner(self) -> E {
        self.0
    }
}

impl<E> From<E> for InternalServerError<E>
where
    E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        InternalServerError(err)
    }
}

impl<E> fmt::Display for InternalServerError<E>
where
    E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<E> HttpError for InternalServerError<E>
where
    E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    type Body = String;

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn to_response(&self, _: &Request<()>) -> Response<Self::Body> {
        let mut response = Response::new(self.to_string());
        *response.status_mut() = self.status_code();
        response
    }
}

// ==== Error ====

type AnyObj = dyn Any + Send + Sync + 'static;

/// A type which holds a value of `HttpError` in a type-erased form.
pub struct Error {
    inner: Box<AnyObj>,
    fmt_debug_fn: fn(&AnyObj, &mut fmt::Formatter<'_>) -> fmt::Result,
    fmt_display_fn: fn(&AnyObj, &mut fmt::Formatter<'_>) -> fmt::Result,
    status_code_fn: fn(&AnyObj) -> StatusCode,
    to_response_fn: fn(&AnyObj, &Request<()>) -> Response<Bytes>,
}

impl<E: HttpError> From<E> for Error {
    fn from(err: E) -> Self {
        Self::new(err)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugInner<'a>(&'a Error);

        impl<'a> fmt::Debug for DebugInner<'a> {
            #[inline]
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                (self.0.fmt_debug_fn)(&*self.0.inner, formatter)
            }
        }

        formatter
            .debug_struct("Error")
            .field("inner", &DebugInner(self))
            .field("fmt_debug_fn", &"<fn>")
            .field("fmt_display_fn", &"<fn>")
            .field("into_response_fn", &"<fn>")
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.fmt_display_fn)(&*self.inner, formatter)
    }
}

impl IntoResponse for Error {
    type Body = Bytes;

    fn into_response(self, request: &Request<()>) -> Response<Self::Body> {
        let mut response = (self.to_response_fn)(&*self.inner, request);
        response.extensions_mut().insert(self);
        response
    }
}

impl Error {
    pub fn new<E>(err: E) -> Self
    where
        E: HttpError,
    {
        fn fmt_debug<E>(this: &AnyObj, f: &mut fmt::Formatter<'_>) -> fmt::Result
        where
            E: HttpError,
        {
            let this = this.downcast_ref::<E>().expect("invalid type id");
            fmt::Debug::fmt(this, f)
        }

        fn fmt_display<E>(this: &AnyObj, f: &mut fmt::Formatter<'_>) -> fmt::Result
        where
            E: HttpError,
        {
            let this = this.downcast_ref::<E>().expect("invalid type id");
            fmt::Display::fmt(this, f)
        }

        fn stauts_code<E>(this: &AnyObj) -> StatusCode
        where
            E: HttpError,
        {
            let this = this.downcast_ref::<E>().expect("invalid type id");
            this.status_code()
        }

        fn to_response<E>(this: &AnyObj, request: &Request<()>) -> Response<Bytes>
        where
            E: HttpError,
        {
            match this.downcast_ref::<E>() {
                Some(err) => err.to_response(request).map(Into::into),
                None => {
                    let msg = Bytes::from_static(b"failed to retrive the original error response");
                    let mut response = Response::new(msg);
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    response
                }
            }
        }

        Self {
            inner: Box::new(err),
            fmt_debug_fn: fmt_debug::<E>,
            fmt_display_fn: fmt_display::<E>,
            status_code_fn: stauts_code::<E>,
            to_response_fn: to_response::<E>,
        }
    }

    /// Returns `true` if the type of contained value is the same as `T`.
    pub fn is<T: HttpError>(&self) -> bool {
        self.inner.is::<T>()
    }

    /// Attempts to downcast the boxed value to a conrete type by reference.
    pub fn downcast_ref<T: HttpError>(&self) -> Option<&T> {
        self.inner.downcast_ref()
    }

    /// Attempts to downcast the boxed value to a conrete type by mutable reference.
    pub fn downcast_mut<T: HttpError>(&mut self) -> Option<&mut T> {
        self.inner.downcast_mut()
    }

    /// Attempts to downcast the boxed value to a conrete type.
    pub fn downcast<T: HttpError>(self) -> Result<T> {
        match <Box<dyn Any>>::downcast::<T>(self.inner) {
            Ok(err) => Ok(*err),
            Err(inner) => Err(Self {
                inner: unsafe {
                    Box::from_raw(
                        Box::into_raw(inner) as *mut AnyObj, // reapply Send + Sync marker
                    )
                },
                ..self
            }),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        (self.status_code_fn)(&*self.inner)
    }
}

/// A type alias of `Result<T, E>` whose error type is restricted to `Error`.
pub type Result<T> = std::result::Result<T, Error>;
