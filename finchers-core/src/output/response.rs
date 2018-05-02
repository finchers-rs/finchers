use either::Either;
use http::StatusCode;
use http::header::{HeaderMap, HeaderValue};

/// Trait representing additional information for constructing an HTTP response.
///
/// This trait is used as a helper to define the implementation of `Responder`.
pub trait HttpResponse {
    /// Returns a HTTP status code.
    fn status_code(&self) -> StatusCode;

    /// Append header values to given header map.
    #[allow(unused_variables)]
    fn append_headers(&self, headers: &mut HeaderMap<HeaderValue>) {}
}

macro_rules! impl_status {
    ($($t:ty),*) => {$(
        impl HttpResponse for $t {
            fn status_code(&self) -> StatusCode {
                StatusCode::OK
            }
        }
    )*};
}

impl_status!(
    bool,
    char,
    f32,
    f64,
    String,
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize
);

impl<L, R> HttpResponse for Either<L, R>
where
    L: HttpResponse,
    R: HttpResponse,
{
    fn status_code(&self) -> StatusCode {
        match *self {
            Either::Left(ref l) => l.status_code(),
            Either::Right(ref r) => r.status_code(),
        }
    }

    fn append_headers(&self, headers: &mut HeaderMap<HeaderValue>) {
        match *self {
            Either::Left(ref l) => l.append_headers(headers),
            Either::Right(ref r) => r.append_headers(headers),
        }
    }
}
