#![allow(missing_docs)]

use {
    crate::error::{Error, HttpError},
    failure::Fail,
    http::StatusCode,
    percent_encoding::percent_decode,
    std::{
        borrow::Cow,
        fmt, net,
        path::PathBuf,
        str::{self, FromStr, Utf8Error},
    },
};

/// A percent-encoded string.
#[repr(C)]
pub struct EncodedStr([u8]);

impl fmt::Debug for EncodedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EncodedStr").field(&&self.0).finish()
    }
}

impl AsRef<[u8]> for EncodedStr {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl PartialEq<str> for EncodedStr {
    fn eq(&self, other: &str) -> bool {
        self.0 == *other.as_bytes()
    }
}

impl<'a, 'b> PartialEq<&'b EncodedStr> for &'a EncodedStr {
    fn eq(&self, other: &&'b EncodedStr) -> bool {
        self.0 == *other.as_bytes()
    }
}

impl<'a> PartialEq<str> for &'a EncodedStr {
    fn eq(&self, other: &str) -> bool {
        (*self).eq(other)
    }
}

impl PartialEq<String> for EncodedStr {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other.as_bytes()
    }
}

impl<'a> PartialEq<String> for &'a EncodedStr {
    fn eq(&self, other: &String) -> bool {
        (*self).eq(other)
    }
}

impl EncodedStr {
    /// Create a new instance of `EncodedStr` from an encoded `str`.
    ///
    /// # Safety
    /// The given string must be a percent-encoded sequence.
    #[inline(always)]
    pub unsafe fn new_unchecked(s: &(impl AsRef<[u8]> + ?Sized)) -> &EncodedStr {
        &*((*s).as_ref() as *const [u8] as *const EncodedStr)
    }

    /// Return the reference to the underling `[u8]` of this value.
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Decode this encoded string as an UTF-8 string.
    #[inline]
    pub fn percent_decode(&self) -> Result<Cow<'_, str>, Utf8Error> {
        percent_decode(&self.0).decode_utf8()
    }

    /// Decode this encoded string as an UTF-8 string.
    ///
    /// This method will not fail and the invalid UTF-8 characters will be
    /// replaced to � (U+FFFD).
    #[inline]
    pub fn percent_decode_lossy(&self) -> Cow<'_, str> {
        percent_decode(&self.0).decode_utf8_lossy()
    }
}

/// Trait representing the conversion from an encoded string.
pub trait FromEncodedStr: Sized + 'static {
    /// The error type which will be returned from `from_encoded_str`.
    type Error: Into<Error>;

    /// Converts an `EncodedStr` to a value of `Self`.
    fn from_encoded_str(s: &EncodedStr) -> Result<Self, Self::Error>;
}

macro_rules! impl_from_segment_from_str {
    ($($t:ty,)*) => {$(
        impl FromEncodedStr for $t {
            type Error = Error;

            #[inline]
            fn from_encoded_str(s: &EncodedStr) -> Result<Self, Self::Error> {
                let s = s.percent_decode().map_err(|cause| DecodeEncodedStrError{cause})?;
                Ok(FromStr::from_str(&*s).map_err(|cause| ParseEncodedStrError{cause})?)
            }
        }
    )*};
}

impl_from_segment_from_str! {
    bool, f32, f64,
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
    net::IpAddr,
    net::Ipv4Addr,
    net::Ipv6Addr,
    net::SocketAddr,
    net::SocketAddrV4,
    net::SocketAddrV6,
}

impl FromEncodedStr for String {
    type Error = DecodeEncodedStrError;

    #[inline]
    fn from_encoded_str(s: &EncodedStr) -> Result<Self, Self::Error> {
        s.percent_decode()
            .map(Cow::into_owned)
            .map_err(|cause| DecodeEncodedStrError { cause })
    }
}

impl FromEncodedStr for PathBuf {
    type Error = DecodeEncodedStrError;

    #[inline]
    fn from_encoded_str(s: &EncodedStr) -> Result<Self, Self::Error> {
        s.percent_decode()
            .map(|s| std::path::PathBuf::from(s.into_owned()))
            .map_err(|cause| DecodeEncodedStrError { cause })
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[fail(display = "failed to decode a percent encoded string to UTF-8")]
pub struct DecodeEncodedStrError {
    #[fail(source)]
    cause: Utf8Error,
}

impl HttpError for DecodeEncodedStrError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[fail(display = "{}", cause)]
pub struct ParseEncodedStrError<E>
where
    E: Fail + Send + Sync + 'static,
{
    #[fail(cause)]
    cause: E,
}

impl<E> HttpError for ParseEncodedStrError<E>
where
    E: Fail + Send + Sync + 'static,
{
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
