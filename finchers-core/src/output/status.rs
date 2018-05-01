use either::Either;
use http::StatusCode;

/// [unstable]
/// Trait representing the HTTP status associated with types.
///
/// This trait is used as a helper to define the implementation of "Responder".
pub trait HttpStatus {
    /// Returns a HTTP status code associated with this type
    fn status_code(&self) -> StatusCode;
}

macro_rules! impl_status {
    ($($t:ty),*) => {$(
        impl HttpStatus for $t {
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

impl<L, R> HttpStatus for Either<L, R>
where
    L: HttpStatus,
    R: HttpStatus,
{
    fn status_code(&self) -> StatusCode {
        match *self {
            Either::Left(ref l) => l.status_code(),
            Either::Right(ref r) => r.status_code(),
        }
    }
}
