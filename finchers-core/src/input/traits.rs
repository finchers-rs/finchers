use std::mem::PinMut;
use std::net;
use std::str::{FromStr, Utf8Error};

use bytes::Bytes;
use failure::Fail;
use http::StatusCode;

use crate::error::{Failure, HttpError, Never};

use super::segments::{EncodedStr, Segment};
use super::Input;

/// Trait representing the transformation from a message body.
pub trait FromBody: 'static + Sized {
    /// The error type which will be returned from `from_data`.
    type Error;

    /// Returns whether the incoming request matches to this type or not.
    #[allow(unused_variables)]
    fn is_match(input: PinMut<Input>) -> bool {
        true
    }

    /// Performs conversion from raw bytes into itself.
    fn from_body(body: Bytes, input: PinMut<Input>) -> Result<Self, Self::Error>;
}

impl FromBody for Bytes {
    type Error = Never;

    fn from_body(body: Bytes, _: PinMut<Input>) -> Result<Self, Self::Error> {
        Ok(body)
    }
}

impl FromBody for String {
    type Error = Failure;

    fn from_body(body: Bytes, _: PinMut<Input>) -> Result<Self, Self::Error> {
        String::from_utf8(body.to_vec())
            .map_err(|cause| Failure::new(StatusCode::BAD_REQUEST, cause))
    }
}

/// Trait representing the conversion from "Segment".
pub trait FromSegment: 'static + Sized {
    /// The error type returned from "from_segment".
    type Error;

    /// Perform conversion from "Segment" to "Self".
    fn from_segment(segment: Segment) -> Result<Self, Self::Error>;
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
pub enum FromSegmentError<E: Fail> {
    #[fail(display = "{}", cause)]
    Decode { cause: Utf8Error },

    #[fail(display = "{}", cause)]
    Parse { cause: E },
}

impl<E: Fail> HttpError for FromSegmentError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

macro_rules! impl_from_segment_from_str {
    ($($t:ty,)*) => {$(
        impl FromSegment for $t {
            type Error = FromSegmentError<<$t as FromStr>::Err>;

            #[inline]
            fn from_segment(segment: Segment) -> Result<Self, Self::Error> {
                let s = segment.as_encoded_str().percent_decode().map_err(|cause| FromSegmentError::Decode{cause})?;
                FromStr::from_str(&*s).map_err(|cause| FromSegmentError::Parse{cause})
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

impl FromSegment for String {
    type Error = Never;

    #[inline]
    fn from_segment(segment: Segment) -> Result<Self, Self::Error> {
        Ok(segment.as_encoded_str().percent_decode_lossy().into_owned())
    }
}

/*
/// Trait representing the conversion from a `Segments`
pub trait FromSegments: 'static + Sized {
    /// The error type returned from `from_segments`
    type Error;

    /// Perform conversion from `Segments` to `Self`.
    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error>;
}

impl<T: FromSegment> FromSegments for Vec<T> {
    type Error = T::Error;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        segments.into_iter().map(|s| T::from_segment(s)).collect()
    }
}

impl FromSegments for String {
    type Error = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        let s = segments.remaining_path().to_owned();
        let _ = segments.last();
        Ok(s)
    }
}

impl FromSegments for PathBuf {
    type Error = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        let s = PathBuf::from(segments.remaining_path());
        let _ = segments.last();
        Ok(s)
    }
}

impl<T: FromSegments> FromSegments for Option<T> {
    type Error = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        Ok(T::from_segments(segments).ok())
    }
}

impl<T: FromSegments> FromSegments for Result<T, T::Error> {
    type Error = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        Ok(T::from_segments(segments))
    }
}
*/

/// Trait representing the transformation from a set of HTTP query.
pub trait FromQuery: Sized + 'static {
    /// The error type which will be returned from `from_query`.
    type Error;

    /// Perform transformation from `QueryItems` into `Self`.
    fn from_query(query: QueryItems) -> Result<Self, Self::Error>;
}

/// An iterator over the elements of query items.
#[derive(Debug)]
pub struct QueryItems<'a> {
    input: &'a [u8],
}

impl<'a> QueryItems<'a> {
    /// Create a new `QueryItems` from a slice of bytes.
    ///
    /// The input must be a valid HTTP query.
    pub fn new<S: AsRef<[u8]> + ?Sized>(input: &'a S) -> QueryItems<'a> {
        QueryItems {
            input: input.as_ref(),
        }
    }

    /// Returns a slice of bytes which contains the remaining query items.
    #[inline(always)]
    pub fn as_slice(&self) -> &'a [u8] {
        self.input
    }
}

// FIXME: return an error if the input is invalid query sequence.
impl<'a> Iterator for QueryItems<'a> {
    type Item = (&'a EncodedStr, &'a EncodedStr);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.input.is_empty() {
                return None;
            }

            let mut s = self.input.splitn(2, |&b| b == b'&');
            let seq = s.next().unwrap();
            self.input = s.next().unwrap_or(&[]);
            if seq.is_empty() {
                continue;
            }

            let mut s = seq.splitn(2, |&b| b == b'=');
            let name = s.next().unwrap();
            let value = s.next().unwrap_or(&[]);
            break unsafe {
                Some((
                    EncodedStr::new_unchecked(name),
                    EncodedStr::new_unchecked(value),
                ))
            };
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        let result = FromSegments::from_segments(&mut segments);
        assert_eq!(result, Ok(PathBuf::from("foo/bar.txt")));
    }
}
*/
