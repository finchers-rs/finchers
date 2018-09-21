use std::marker::PhantomData;
use std::rc::Rc;

use crate::input::{EncodedStr, Input};

#[derive(Debug)]
pub(crate) struct Cursor {
    pub(crate) pos: usize,
    pub(crate) popped: usize,
}

impl Cursor {
    fn clone(&self) -> Cursor {
        Cursor { ..*self }
    }
}

/// An iterator over the remaining path segments.
#[derive(Debug)]
pub struct Context<'a> {
    input: &'a mut Input,
    cursor: Cursor,
    _marker: PhantomData<Rc<()>>,
}

impl<'a> Context<'a> {
    #[doc(hidden)]
    #[inline]
    pub fn new(input: &'a mut Input) -> Context<'a> {
        Context {
            input,
            cursor: Cursor { pos: 1, popped: 0 },
            _marker: PhantomData,
        }
    }

    /// Returns the pinned reference to the value of `Input`.
    #[inline]
    pub fn input(&mut self) -> &mut Input {
        &mut *self.input
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

    pub(crate) fn clone_reborrowed<'b>(&'b mut self) -> Context<'b>
    where
        'a: 'b,
    {
        Context {
            input: &mut *self.input,
            cursor: self.cursor.clone(),
            _marker: PhantomData,
        }
    }

    /// Returns the cursor position in the original path
    #[inline]
    pub(crate) fn current_cursor(&self) -> Cursor {
        self.cursor.clone()
    }

    pub(crate) fn reset_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::body::ReqBody;
    use http::Request;

    #[test]
    fn test_segments() {
        let request = Request::get("/foo/bar.txt")
            .body(ReqBody::from_hyp(Default::default()))
            .unwrap();
        let mut input = Input::new(request);
        let mut ecx = Context::new(&mut input);

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
        let request = Request::get("/")
            .body(ReqBody::from_hyp(Default::default()))
            .unwrap();
        let mut input = Input::new(request);
        let mut ecx = Context::new(&mut input);

        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next_segment().is_none());
    }
}
