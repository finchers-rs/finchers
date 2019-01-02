//! The definition of contextual information during applying endpoints.

use std::marker::PhantomData;
use std::rc::Rc;

use crate::input::{EncodedStr, Input};

#[derive(Debug, Clone)]
pub(crate) struct Cursor {
    pos: usize,
    popped: usize,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor { pos: 1, popped: 0 }
    }
}

impl Cursor {
    pub(crate) fn popped(&self) -> usize {
        self.popped
    }
}

/// The contextual information during calling `Endpoint::apply()`.
///
/// This type behaves an iterator over the remaining path segments.
#[derive(Debug)]
pub struct ApplyContext<'a, Bd> {
    input: &'a mut Input<Bd>,
    cursor: &'a mut Cursor,
    _marker: PhantomData<Rc<()>>,
}

impl<'a, Bd> ApplyContext<'a, Bd> {
    #[inline]
    pub(crate) fn new(input: &'a mut Input<Bd>, cursor: &'a mut Cursor) -> Self {
        ApplyContext {
            input,
            cursor,
            _marker: PhantomData,
        }
    }

    /// Returns a mutable reference to the value of `Input`.
    #[inline]
    pub fn input(&mut self) -> &mut Input<Bd> {
        &mut *self.input
    }

    pub(crate) fn cursor(&mut self) -> &mut Cursor {
        &mut *self.cursor
    }

    /// Returns the remaining path in this segments
    #[inline]
    pub fn remaining_path(&self) -> &EncodedStr {
        unsafe { EncodedStr::new_unchecked(&self.input.uri().path()[self.cursor.pos..]) }
    }

    /// Advances the cursor and returns the next segment.
    #[inline]
    pub fn next_segment(&mut self) -> Option<&EncodedStr> {
        let path = &self.input.uri().path();
        if self.cursor.pos == path.len() {
            return None;
        }

        let s = if let Some(offset) = path[self.cursor.pos..].find('/') {
            let s = &path[self.cursor.pos..(self.cursor.pos + offset)];
            self.cursor.pos += offset + 1;
            self.cursor.popped += 1;
            s
        } else {
            let s = &path[self.cursor.pos..];
            self.cursor.pos = path.len();
            self.cursor.popped += 1;
            s
        };

        Some(unsafe { EncodedStr::new_unchecked(s) })
    }
}

impl<'a, Bd> std::ops::Deref for ApplyContext<'a, Bd> {
    type Target = Input<Bd>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.input
    }
}

impl<'a, Bd> std::ops::DerefMut for ApplyContext<'a, Bd> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.input()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;

    #[test]
    fn test_segments() {
        let request = Request::get("/foo/bar.txt").body(()).unwrap();
        let mut input = Input::new(request);
        let mut cursor = Cursor::default();
        let mut ecx = ApplyContext::new(&mut input, &mut cursor);

        assert_eq!(ecx.remaining_path(), "foo/bar.txt");
        assert_eq!(ecx.next_segment().map(|s| s.as_bytes()), Some(&b"foo"[..]));
        assert_eq!(ecx.remaining_path(), "bar.txt");
        assert_eq!(
            ecx.next_segment().map(|s| s.as_bytes()),
            Some(&b"bar.txt"[..])
        );
        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next_segment().is_none());
        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next_segment().is_none());
    }

    #[test]
    fn test_segments_from_root_path() {
        let request = Request::get("/").body(()).unwrap();
        let mut input = Input::new(request);
        let mut cursor = Cursor::default();
        let mut ecx = ApplyContext::new(&mut input, &mut cursor);

        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next_segment().is_none());
    }
}
