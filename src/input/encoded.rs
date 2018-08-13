use std::borrow::Cow;
use std::fmt;
use std::net;
use std::path::PathBuf;
use std::str::{self, FromStr, Utf8Error};

use failure::Fail;
use http::StatusCode;
use percent_encoding::percent_decode;

use error::HttpError;

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

    /// Decode this encoded string as an UTF-8 string.
    ///
    /// This method will replace the plus ('+') character with a half-width space
    /// before decoding.
    #[inline]
    pub fn url_decode(&self) -> Result<Cow<'_, str>, Utf8Error> {
        let replaced = replace_plus(&self.0);
        let v = match percent_decode(&*replaced).if_any() {
            Some(v) => v,
            None => match replaced {
                Cow::Borrowed(b) => return str::from_utf8(b).map(Cow::Borrowed),
                Cow::Owned(v) => v,
            },
        };
        String::from_utf8(v)
            .map(Cow::Owned)
            .map_err(|e| e.utf8_error())
    }
}

fn replace_plus(input: &[u8]) -> Cow<'_, [u8]> {
    match input.iter().position(|&b| b == b'+') {
        None => Cow::Borrowed(input),
        Some(pos) => {
            let mut replaced = input.to_owned();
            replaced[pos] = b' ';
            replaced[pos + 1..].iter_mut().for_each(|b| {
                if *b == b'+' {
                    *b = b' ';
                }
            });
            Cow::Owned(replaced)
        }
    }
}

/// Trait representing the conversion from an encoded string.
pub trait FromEncodedStr: Sized + 'static {
    /// The error type which will be returned from `from_encoded_str`.
    type Error;

    /// Converts an `EncodedStr` to a value of `Self`.
    fn from_encoded_str(s: &EncodedStr) -> Result<Self, Self::Error>;
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
pub enum FromEncodedStrError<E: Fail> {
    #[fail(display = "{}", cause)]
    Decode { cause: Utf8Error },
    #[fail(display = "{}", cause)]
    Parse { cause: E },
}

impl<E: Fail> HttpError for FromEncodedStrError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

macro_rules! impl_from_segment_from_str {
    ($($t:ty,)*) => {$(
        impl FromEncodedStr for $t {
            type Error = FromEncodedStrError<<$t as FromStr>::Err>;

            #[inline]
            fn from_encoded_str(s: &EncodedStr) -> Result<Self, Self::Error> {
                let s = s.percent_decode().map_err(|cause| FromEncodedStrError::Decode{cause})?;
                FromStr::from_str(&*s).map_err(|cause| FromEncodedStrError::Parse{cause})
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
    type Error = Utf8Error;

    #[inline]
    fn from_encoded_str(s: &EncodedStr) -> Result<Self, Self::Error> {
        s.percent_decode().map(Cow::into_owned)
    }
}

impl FromEncodedStr for PathBuf {
    type Error = Utf8Error;

    #[inline]
    fn from_encoded_str(s: &EncodedStr) -> Result<Self, Self::Error> {
        s.percent_decode().map(|s| PathBuf::from(s.into_owned()))
    }
}
