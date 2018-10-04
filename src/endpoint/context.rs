//! The definition of contextual information during applying endpoints.

use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::rc::Rc;

use input::{EncodedStr, Input};

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
pub struct ApplyContext<'a> {
    input: &'a mut Input,
    cursor: &'a mut Cursor,
    _marker: PhantomData<Rc<()>>,
}

impl<'a> ApplyContext<'a> {
    #[inline]
    pub(crate) fn new(input: &'a mut Input, cursor: &'a mut Cursor) -> ApplyContext<'a> {
        ApplyContext {
            input,
            cursor,
            _marker: PhantomData,
        }
    }

    /// Returns a mutable reference to the value of `Input`.
    #[inline]
    pub fn input(&mut self) -> &mut Input {
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

impl<'a> Deref for ApplyContext<'a> {
    type Target = Input;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.input
    }
}

impl<'a> DerefMut for ApplyContext<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.input()
    }
}

/// The contexual information per request during polling the future returned from endpoints.
///
/// The value of this context can be indirectly access by calling `with_get_cx()`.
#[derive(Debug)]
pub struct TaskContext<'a> {
    input: &'a mut Input,
    cursor: &'a Cursor,
    _marker: PhantomData<Rc<()>>,
}

impl<'a> TaskContext<'a> {
    pub(crate) fn new(input: &'a mut Input, cursor: &'a Cursor) -> TaskContext<'a> {
        TaskContext {
            input,
            cursor,
            _marker: PhantomData,
        }
    }

    #[allow(missing_docs)]
    pub fn input(&mut self) -> &mut Input {
        &mut *self.input
    }
}

impl<'a> Deref for TaskContext<'a> {
    type Target = Input;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.input
    }
}

impl<'a> DerefMut for TaskContext<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.input()
    }
}

thread_local!(static CX: Cell<Option<NonNull<TaskContext<'static>>>> = Cell::new(None));

struct SetOnDrop(Option<NonNull<TaskContext<'static>>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        CX.with(|cx| cx.set(self.0));
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(cast_ptr_alignment))]
pub(crate) fn with_set_cx<R>(current: &mut TaskContext<'_>, f: impl FnOnce() -> R) -> R {
    CX.with(|cx| {
        cx.set(Some(unsafe {
            NonNull::new_unchecked(
                current as *mut TaskContext<'_> as *mut () as *mut TaskContext<'static>,
            )
        }))
    });
    let _reset = SetOnDrop(None);
    f()
}

/// Acquires a mutable reference to `TaskContext` from the current task context
/// and executes the provided function using its value.
///
/// This function is usually used to access the value of `Input` within the `Future`
/// returned by the `Endpoint`.
///
/// # Panics
///
/// A panic will occur if you call this function inside the provided closure `f`, since the
/// reference to `TaskContext` on the task context is invalidated while executing `f`.
pub fn with_get_cx<R>(f: impl FnOnce(&mut TaskContext<'_>) -> R) -> R {
    let prev = CX.with(|cx| cx.replace(None));
    let _reset = SetOnDrop(prev);
    match prev {
        Some(mut ptr) => unsafe { f(ptr.as_mut()) },
        None => panic!("The reference to TaskContext is not set at the current context."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;
    use input::ReqBody;

    #[test]
    fn test_segments() {
        let request = Request::get("/foo/bar.txt")
            .body(ReqBody::new(Default::default()))
            .unwrap();
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
        let request = Request::get("/")
            .body(ReqBody::new(Default::default()))
            .unwrap();
        let mut input = Input::new(request);
        let mut cursor = Cursor::default();
        let mut ecx = ApplyContext::new(&mut input, &mut cursor);

        assert_eq!(ecx.remaining_path(), "");
        assert!(ecx.next_segment().is_none());
    }
}
