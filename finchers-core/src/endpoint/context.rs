use Input;
use percent_encoding::percent_decode;
use std::borrow::Cow;
use std::fmt;
use std::ops::Range;
use std::str::Utf8Error;

/// A context during the routing.
#[derive(Debug, Clone)]
pub struct Context<'a> {
    input: &'a Input,
    segments: Segments<'a>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(input: &'a Input) -> Self {
        Context {
            input: input,
            segments: Segments::from(input.request().uri().path()),
        }
    }

    /// Return the reference to `Input`.
    pub fn input(&self) -> &'a Input {
        self.input
    }

    /// Return the reference to the instance of `Segments`.
    pub fn segments(&mut self) -> &mut Segments<'a> {
        &mut self.segments
    }
}

/// An iterator over the remaining path segments.
#[derive(Debug, Copy, Clone)]
pub struct Segments<'a> {
    path: &'a str,
    pos: usize,
    popped: usize,
}

impl<'a> From<&'a str> for Segments<'a> {
    fn from(path: &'a str) -> Self {
        debug_assert!(!path.is_empty());
        debug_assert_eq!(path.chars().next(), Some('/'));
        Segments {
            path,
            pos: 1,
            popped: 0,
        }
    }
}

impl<'a> Segments<'a> {
    /// Returns the remaining path in this segments
    #[inline]
    pub fn remaining_path(&self) -> &'a str {
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
}

impl<'a> Iterator for Segments<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.path.len() {
            return None;
        }
        if let Some(offset) = self.path[self.pos..].find('/') {
            let segment = Segment {
                s: self.path,
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
                s: self.path,
                range: Range {
                    start: self.pos,
                    end: self.path.len(),
                },
            };
            self.pos = self.path.len();
            self.popped += 1;
            Some(segment)
        }
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
    pub fn as_encoded_str(&self) -> EncodedStr<'a> {
        unsafe { EncodedStr::new_unchecked(&self.s[self.range.clone()]) }
    }

    /// Returns the range of this segment in the original path.
    #[inline]
    pub fn as_range(&self) -> Range<usize> {
        self.range.clone()
    }
}

/// A percent-encoded string.
#[repr(C)]
pub struct EncodedStr<'a>(&'a str);

impl<'a> fmt::Debug for EncodedStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("EncodedStr").field(&self.0).finish()
    }
}

impl<'a> EncodedStr<'a> {
    /// Create a new instance of `EncodedStr` from an encoded `str`.
    ///
    /// # Safety
    /// The given string must be a percent-encoded sequence.
    #[inline(always)]
    pub unsafe fn new_unchecked(s: &'a str) -> EncodedStr<'a> {
        EncodedStr(s)
    }

    /// Return the reference to the underling `str` of this value.
    #[inline(always)]
    pub fn as_raw(&self) -> &'a str {
        self.0
    }

    /// Decode this encoded string as an UTF-8 string.
    #[inline]
    pub fn decode_utf8(&self) -> Result<Cow<'a, str>, Utf8Error> {
        percent_decode(self.0.as_bytes()).decode_utf8()
    }

    /// Decode this encoded string as an UTF-8 string.
    ///
    /// This method will not fail and the invalid UTF-8 characters will be
    /// replaced to � (U+FFFD).
    #[inline]
    pub fn decode_utf8_lossy(&self) -> Cow<'a, str> {
        percent_decode(self.0.as_bytes()).decode_utf8_lossy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        assert_eq!(segments.remaining_path(), "foo/bar.txt");
        assert_eq!(segments.next().map(|s| s.as_encoded_str().as_raw()), Some("foo"));
        assert_eq!(segments.remaining_path(), "bar.txt");
        assert_eq!(segments.next().map(|s| s.as_encoded_str().as_raw()), Some("bar.txt"));
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_encoded_str().as_raw()), None);
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_encoded_str().as_raw()), None);
    }

    #[test]
    fn test_segments_from_root_path() {
        let mut segments = Segments::from("/");
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_encoded_str().as_raw()), None);
    }

}
