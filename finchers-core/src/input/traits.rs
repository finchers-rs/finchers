use std::mem::PinMut;

use bytes::Bytes;
use http::StatusCode;

use error::{Failure, Never};

use super::encoded::EncodedStr;
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
