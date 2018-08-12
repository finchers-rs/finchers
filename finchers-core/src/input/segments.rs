use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Range;
use std::rc::Rc;
use std::str;
use std::str::Utf8Error;

use percent_encoding::percent_decode;

/// An iterator over the remaining path segments.
#[derive(Debug, Clone)]
pub struct Cursor {
    path: &'static str,
    pos: usize,
    popped: usize,
    _marker: PhantomData<Rc<()>>,
}

impl Cursor {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn new(path: &str) -> Self {
        // change the lifetime of path string.
        //
        // safety:
        // * The lifetime of provided str ('a) outlives this instance.
        //   - It implies that the instance of `Input` which holding `path`
        //     does not move until the instance of `Self` dies.
        let path = &*(path as *const str);

        Cursor {
            path,
            pos: 1,
            popped: 0,
            _marker: PhantomData,
        }
    }

    /// Returns the remaining path in this segments
    #[inline]
    pub unsafe fn remaining_path(&self) -> &str {
        &self.path[self.pos..]
    }

    /// Returns the cursor position in the original path
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the number of segments already popped
    #[inline]
    pub fn popped(&self) -> usize {
        self.popped
    }

    #[allow(missing_docs)]
    pub unsafe fn next_segment<'a>(&'a mut self) -> Option<Segment<'a>> {
        let path = &self.path;
        if self.pos == path.len() {
            return None;
        }

        if let Some(offset) = path[self.pos..].find('/') {
            let segment = Segment {
                s: path,
                range: Range {
                    start: self.pos,
                    end: self.pos + offset,
                },
            };
            self.pos += offset + 1;
            self.popped += 1;
            Some(segment)
        } else {
            let segment = Segment {
                s: path,
                range: Range {
                    start: self.pos,
                    end: path.len(),
                },
            };
            self.pos = path.len();
            self.popped += 1;
            Some(segment)
        }
    }

    #[allow(missing_docs)]
    pub unsafe fn consume_all_segments(&mut self) {
        while let Some(..) = self.next_segment() {}
    }
}

/// A path segment in the HTTP requests.
#[derive(Debug, Clone)]
pub struct Segment<'a> {
    s: &'a str,
    range: Range<usize>,
}

impl<'a> Segment<'a> {
    /// Create a `Segment` from a pair of path string and the range of segment.
    pub fn new(s: &'a str, range: Range<usize>) -> Segment<'a> {
        Segment { s, range }
    }

    /// Return an `EncodedStr` from this segment.
    pub fn as_encoded_str(&self) -> &'a EncodedStr {
        unsafe { EncodedStr::new_unchecked(self.s[self.range.clone()].as_bytes()) }
    }

    /// Returns the range of this segment in the original path.
    #[inline]
    pub fn as_range(&self) -> Range<usize> {
        self.range.clone()
    }
}

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

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        assert_eq!(segments.remaining_path(), "foo/bar.txt");
        assert_eq!(
            segments.next().map(|s| s.as_encoded_str().as_bytes()),
            Some(&b"foo"[..])
        );
        assert_eq!(segments.remaining_path(), "bar.txt");
        assert_eq!(
            segments.next().map(|s| s.as_encoded_str().as_bytes()),
            Some(&b"bar.txt"[..])
        );
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_encoded_str().as_bytes()), None);
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_encoded_str().as_bytes()), None);
    }

    #[test]
    fn test_segments_from_root_path() {
        let mut segments = Segments::from("/");
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_encoded_str().as_bytes()), None);
    }

}
*/
