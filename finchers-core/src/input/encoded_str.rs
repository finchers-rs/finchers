use std::borrow::Cow;
use std::fmt;
use std::str;
use std::str::Utf8Error;

use percent_encoding::percent_decode;

/// A percent-encoded string.
#[repr(C)]
pub struct EncodedStr([u8]);

impl fmt::Debug for EncodedStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("EncodedStr").field(&&self.0).finish()
    }
}

impl AsRef<[u8]> for EncodedStr {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl EncodedStr {
    /// Create a new instance of `EncodedStr` from an encoded `str`.
    ///
    /// # Safety
    /// The given string must be a percent-encoded sequence.
    #[inline(always)]
    pub unsafe fn new_unchecked(s: &[u8]) -> &EncodedStr {
        &*(s as *const [u8] as *const EncodedStr)
    }

    /// Return the reference to the underling `[u8]` of this value.
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Decode this encoded string as an UTF-8 string.
    #[inline]
    pub fn percent_decode(&self) -> Result<Cow<str>, Utf8Error> {
        percent_decode(&self.0).decode_utf8()
    }

    /// Decode this encoded string as an UTF-8 string.
    ///
    /// This method will not fail and the invalid UTF-8 characters will be
    /// replaced to � (U+FFFD).
    #[inline]
    pub fn percent_decode_lossy(&self) -> Cow<str> {
        percent_decode(&self.0).decode_utf8_lossy()
    }

    /// Decode this encoded string as an UTF-8 string.
    ///
    /// This method will replace the plus ('+') character with a half-width space
    /// before decoding.
    #[inline]
    pub fn url_decode(&self) -> Result<Cow<str>, Utf8Error> {
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

fn replace_plus<'a>(input: &'a [u8]) -> Cow<'a, [u8]> {
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
