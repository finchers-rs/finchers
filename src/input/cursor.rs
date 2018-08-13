use std::marker::PhantomData;
use std::rc::Rc;
use std::str;

use super::encoded::EncodedStr;

/// An iterator over the remaining path segments.
#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    path: &'a str,
    pos: usize,
    popped: usize,
    _marker: PhantomData<Rc<()>>,
}

impl<'a> Cursor<'a> {
    #[doc(hidden)]
    #[inline]
    pub fn new(path: &'a str) -> Cursor<'a> {
        Cursor {
            path,
            pos: 1,
            popped: 0,
            _marker: PhantomData,
        }
    }

    #[allow(missing_docs)]
    #[inline(always)]
    pub fn by_ref(&mut self) -> &mut Self {
        self
    }

    /// Returns the remaining path in this segments
    #[inline]
    pub fn remaining_path(&self) -> &'a EncodedStr {
        unsafe { EncodedStr::new_unchecked(&self.path[self.pos..]) }
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

impl<'a> Iterator for Cursor<'a> {
    type Item = &'a EncodedStr;

    fn next(&mut self) -> Option<&'a EncodedStr> {
        let path = &self.path;
        if self.pos == path.len() {
            return None;
        }

        let s = if let Some(offset) = path[self.pos..].find('/') {
            let s = &path[self.pos..(self.pos + offset)];
            self.pos += offset + 1;
            self.popped += 1;
            s
        } else {
            let s = &path[self.pos..];
            self.pos = path.len();
            self.popped += 1;
            s
        };

        Some(unsafe { EncodedStr::new_unchecked(s) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segments() {
        static PATH: &str = "/foo/bar.txt";
        let mut cursor = Cursor::new(PATH);

        assert_eq!(cursor.remaining_path(), "foo/bar.txt");
        assert_eq!(cursor.next().map(|s| s.as_bytes()), Some(&b"foo"[..]));
        assert_eq!(cursor.remaining_path(), "bar.txt");
        assert_eq!(cursor.next().map(|s| s.as_bytes()), Some(&b"bar.txt"[..]));
        assert_eq!(cursor.remaining_path(), "");
        assert!(cursor.next().is_none());
        assert_eq!(cursor.remaining_path(), "");
        assert!(cursor.next().is_none());
    }

    #[test]
    fn test_segments_from_root_path() {
        static PATH: &str = "/";
        let mut cursor = Cursor::new(PATH);

        assert_eq!(cursor.remaining_path(), "");
        assert!(cursor.next().is_none());
    }
}
