#![allow(missing_docs)]

use std::{fmt, mem, str};
use std::ops::Deref;
use bytes::Bytes;

/// A reference counted UTF-8 sequence.
pub struct BytesString(Bytes);

impl fmt::Debug for BytesString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for BytesString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for BytesString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Into<Bytes> for BytesString {
    fn into(self) -> Bytes {
        self.0
    }
}

impl Into<String> for BytesString {
    fn into(self) -> String {
        self.as_str().to_owned()
    }
}

impl BytesString {
    pub fn from_static(s: &'static str) -> BytesString {
        unsafe { Self::from_shared_unchecked(Bytes::from_static(s.as_bytes())) }
    }

    pub fn from_shared(bytes: Bytes) -> Result<BytesString, str::Utf8Error> {
        let _ = str::from_utf8(&*bytes)?;
        Ok(unsafe { Self::from_shared_unchecked(bytes) })
    }

    pub unsafe fn from_shared_unchecked(bytes: Bytes) -> BytesString {
        BytesString(bytes)
    }

    pub fn as_str(&self) -> &str {
        unsafe { mem::transmute::<&[u8], _>(self.0.as_ref()) }
    }

    // TODO: add method creating substrings
}
