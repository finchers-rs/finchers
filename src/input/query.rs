//! Components for parsing query strings.

use failure::{Error, Fail, SyncFailure};
use serde::de;
use serde::de::{DeserializeOwned, IntoDeserializer};
use serde_qs;
use std::fmt;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::Deref;

use super::encoded::EncodedStr;

/// Trait representing the transformation from a set of HTTP query.
pub trait FromQuery: Sized + 'static {
    /// The error type which will be returned from `from_query`.
    type Error: Into<Error>;

    /// Perform transformation from `QueryItems` into `Self`.
    fn from_query(query: QueryItems<'_>) -> Result<Self, Self::Error>;
}

impl<T: FromQuery> FromQuery for Option<T> {
    type Error = T::Error;

    fn from_query(query: QueryItems<'_>) -> Result<Self, Self::Error> {
        if query.as_bytes().is_some() {
            T::from_query(query).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// An iterator over the elements of query items.
#[derive(Debug)]
pub struct QueryItems<'a> {
    input: Option<&'a [u8]>,
}

impl<'a> QueryItems<'a> {
    pub(crate) fn empty() -> QueryItems<'a> {
        QueryItems { input: None }
    }

    pub(crate) unsafe fn new_unchecked<S>(input: &'a S) -> QueryItems<'a>
    where
        S: AsRef<[u8]> + ?Sized,
    {
        QueryItems {
            input: Some(input.as_ref()),
        }
    }

    /// Returns a slice of bytes which contains the remaining query items.
    #[inline(always)]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        self.input
    }
}

// FIXME: return an error if the input is invalid query sequence.
impl<'a> Iterator for QueryItems<'a> {
    type Item = (&'a EncodedStr, &'a EncodedStr);

    fn next(&mut self) -> Option<Self::Item> {
        let input = self.input.as_mut()?;
        loop {
            if input.is_empty() {
                return None;
            }

            let mut s = input.splitn(2, |&b| b == b'&');
            let seq = s.next().unwrap();
            *input = s.next().unwrap_or(&[]);
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

/// A wrapper struct to add the implementation of `FromQuery` to `Deserialize`able types.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Serde<T>(pub T);

impl<T> Serde<T> {
    /// Consume itself and return the inner data of `T`.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Serde<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromQuery for Serde<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = SerdeParseError;

    #[inline]
    fn from_query(items: QueryItems<'_>) -> Result<Self, Self::Error> {
        match items.as_bytes() {
            Some(s) => serde_qs::from_bytes(s)
                .map(Serde)
                .map_err(|cause| SerdeParseError::Parse {
                    cause: SyncFailure::new(cause),
                }),
            None => Err(SerdeParseError::MissingQuery),
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Fail)]
pub enum SerdeParseError {
    #[fail(display = "{}", cause)]
    Parse { cause: SyncFailure<serde_qs::Error> },

    #[fail(display = "missing query")]
    MissingQuery,
}

#[allow(missing_debug_implementations)]
struct CSVSeqVisitor<I, T> {
    _marker: PhantomData<fn() -> (I, T)>,
}

impl<'de, I, T> de::Visitor<'de> for CSVSeqVisitor<I, T>
where
    I: FromIterator<T>,
    T: de::Deserialize<'de>,
{
    type Value = I;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        s.split(',')
            .map(|s| de::Deserialize::deserialize(s.into_deserializer()))
            .collect()
    }
}

/// Deserialize a comma-separated string to a sequence of `T`.
///
/// This function is typically used as the attribute in the derivation of `serde::Deserialize`.
pub fn from_csv<'de, D, I, T>(de: D) -> Result<I, D::Error>
where
    D: de::Deserializer<'de>,
    I: FromIterator<T>,
    T: de::Deserialize<'de>,
{
    de.deserialize_str(CSVSeqVisitor {
        _marker: PhantomData,
    })
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
