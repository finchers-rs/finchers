//! Definition of `EndpointAction` and related components.

use {
    crate::{
        common::Tuple, //
        endpoint::syntax::encoded::EncodedStr,
        error::Error,
        service::Context,
    },
    futures::{Future, Poll},
    std::{marker::PhantomData, rc::Rc},
};

/// An enum representing the result of `EndpointAction::preflight`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Preflight<T> {
    /// The action has been completed with the specified output value.
    Completed(T),

    /// The action is incomplete.
    Incomplete,
}

impl<T> Preflight<T> {
    /// Returns `true` if the value of this is `Completed(x)`.
    pub fn is_completed(&self) -> bool {
        match self {
            Preflight::Completed(..) => true,
            Preflight::Incomplete => false,
        }
    }

    /// Returns `true` if the value of this is `InComplete`.
    pub fn is_incomplete(&self) -> bool {
        !self.is_completed()
    }

    /// Converts the contained value using the specified closure.
    pub fn map<U>(self, op: impl FnOnce(T) -> U) -> Preflight<U> {
        match self {
            Preflight::Completed(x) => Preflight::Completed(op(x)),
            Preflight::Incomplete => Preflight::Incomplete,
        }
    }
}

/// A trait that abstracts the *action* of endpoints.
pub trait EndpointAction<Bd> {
    /// The type returned from this action.
    type Output: Tuple;

    /// Applies an incoming request to this action and returns the result if possible.
    ///
    /// This method is called **only once** after the instance of `Self` is created.
    /// The signature of this method is almost the same as `poll_action`, but there are
    /// the following differences:
    ///
    /// * The return type of this method is **not** `Poll<Self::Output, Self::Error>`.
    ///   When this method returns an `Ok(Incomplete)`, it means that the action needs to poll
    ///   the internal resource or access `ActionContext` to complete itself. Therefore,
    ///   it is not possible to poll any asynchronous objects that this action holds
    ///   within this method.
    /// * The result of this method may affect the behavior of routing. If this method will
    ///   return an `Err`, it is possible that the combinator may ignore this error value and
    ///   choose another `EndpointAction` without aborting the process (in contrast, the error
    ///   values returned from `poll_action` are not ignored).
    ///
    /// Some limitations are added to `PreflightContext` in order to keep consistency when
    /// another endpoint returns an error (for example, it cannot be taken the instance
    /// of request body inside of this method).
    ///
    /// By default, this method does nothing and immediately returns an `Ok(Preflight::Incomplete)`.
    #[allow(unused_variables)]
    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        Ok(Preflight::Incomplete)
    }

    /// Progress this action and returns the result if ready.
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error>;
}

/// A variant of `EndpointAction` that the implementor always returns its
/// result from `preflight`.
#[allow(missing_docs)]
pub trait OneshotAction {
    type Output: Tuple;

    /// Applies an incoming request to this action and returns its result.
    fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error>;

    /// Consume `self` and convert it into an implementor of `EndpointAction`.
    fn into_action(self) -> Oneshot<Self>
    where
        Self: Sized,
    {
        Oneshot(Some(self))
    }
}

/// Wrapper for providing an implementation of `EndpointAction` to `OneshotAction`s.
#[derive(Debug)]
pub struct Oneshot<T>(Option<T>);

impl<T, Bd> EndpointAction<Bd> for Oneshot<T>
where
    T: OneshotAction,
{
    type Output = T::Output;

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        let action = self.0.take().expect("cannot apply twice");
        action.preflight(cx).map(Preflight::Completed)
    }

    fn poll_action(&mut self, _: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        debug_assert!(self.0.is_none());
        unreachable!()
    }
}

impl<F, Bd> EndpointAction<Bd> for F
where
    F: Future,
    F::Item: Tuple,
    F::Error: Into<Error>,
{
    type Output = F::Item;

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        cx.context_mut().set(|| self.poll()).map_err(Into::into)
    }
}

// ==== Context ====

/// A set of contextual values used by `EndpointAction::preflight`.
#[derive(Debug, Clone)]
pub struct PreflightContext<'a> {
    context: &'a Context,
    cursor: CursorInner,
    _anchor: PhantomData<Rc<()>>,
}

impl<'a> PreflightContext<'a> {
    #[inline]
    pub(crate) fn new(context: &'a Context) -> Self {
        PreflightContext {
            context,
            cursor: CursorInner { pos: 1, popped: 0 },
            _anchor: PhantomData,
        }
    }

    /// Returns a reference to the request context.
    #[inline]
    pub fn context(&self) -> &Context {
        &*self.context
    }

    /// Creates a `Cursor` to traverse the path segments.
    #[inline]
    pub fn cursor(&mut self) -> Cursor<'_> {
        Cursor {
            inner: &mut self.cursor,
            path: self.context.uri().path(),
        }
    }
}

impl<'a> std::ops::Deref for PreflightContext<'a> {
    type Target = Context;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.context()
    }
}

/// A proxy type that traverses the path segments.
#[derive(Debug)]
pub struct Cursor<'cx> {
    inner: &'cx mut CursorInner,
    path: &'cx str,
}

#[derive(Debug, Clone)]
struct CursorInner {
    pos: usize,
    popped: usize,
}

impl<'cx> Cursor<'cx> {
    /// Returns the number of segments already popped.
    pub fn num_popped_segments(&self) -> usize {
        self.inner.popped
    }

    /// Advances the inner state and returns the next segment if possible.
    #[inline]
    pub fn next_segment(&mut self) -> Option<&'cx EncodedStr> {
        if self.inner.pos == self.path.len() {
            return None;
        }

        let s = if let Some(offset) = self.path[self.inner.pos..].find('/') {
            let s = &self.path[self.inner.pos..(self.inner.pos + offset)];
            self.inner.pos += offset + 1;
            self.inner.popped += 1;
            s
        } else {
            let s = &self.path[self.inner.pos..];
            self.inner.pos = self.path.len();
            self.inner.popped += 1;
            s
        };

        Some(unsafe { EncodedStr::new_unchecked(s) })
    }

    /// Returns the part of remaining path that is not extracted.
    #[inline]
    pub fn remaining_path(&self) -> &'cx EncodedStr {
        unsafe { EncodedStr::new_unchecked(&self.path[self.inner.pos..]) }
    }
}

impl<'cx> Iterator for Cursor<'cx> {
    type Item = &'cx EncodedStr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_segment()
    }
}

/// A set for contextual values used by `EndpointAction::poll_action`.
#[derive(Debug)]
pub struct ActionContext<'a, Bd> {
    context: &'a mut Context,
    body: &'a mut Option<Bd>,
    _anchor: PhantomData<Rc<()>>,
}

impl<'a, Bd> ActionContext<'a, Bd> {
    pub(crate) fn new(context: &'a mut Context, body: &'a mut Option<Bd>) -> Self {
        Self {
            context,
            body,
            _anchor: PhantomData,
        }
    }

    /// Returns a reference to the request context.
    pub fn context(&self) -> &Context {
        &*self.context
    }

    /// Returns a mutable reference to the request context.
    pub fn context_mut(&mut self) -> &mut Context {
        &mut *self.context
    }

    /// Returns a reference to the instance of request body if exists.
    pub fn body(&self) -> Option<&Bd> {
        self.body.as_ref()
    }

    /// Returns a mutable reference to the instance of request body if exists.
    pub fn body_mut(&mut self) -> Option<&mut Bd> {
        self.body.as_mut()
    }

    /// Takes the instance of request body from this context.
    ///
    /// This method will return an `Err` if the body has already taken by someone.
    pub fn take_body(&mut self) -> Result<Bd, Error> {
        self.body.take().ok_or_else(|| {
            crate::error::internal_server_error(
                "the request body has already been stolen by someone",
            )
        })
    }
}

impl<'a, Bd> std::ops::Deref for ActionContext<'a, Bd> {
    type Target = Context;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.context()
    }
}

impl<'a, Bd> std::ops::DerefMut for ActionContext<'a, Bd> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;

    #[test]
    fn test_segments() {
        let request = Request::get("/foo/bar.txt").body(()).unwrap();
        let context = Context::new(request);
        let mut ecx = PreflightContext::new(&context);

        assert_eq!(ecx.cursor().remaining_path(), "foo/bar.txt");
        assert_eq!(ecx.cursor().next().map(|s| s.as_bytes()), Some(&b"foo"[..]));
        assert_eq!(ecx.cursor().remaining_path(), "bar.txt");
        assert_eq!(
            ecx.cursor().next().map(|s| s.as_bytes()),
            Some(&b"bar.txt"[..])
        );
        assert_eq!(ecx.cursor().remaining_path(), "");
        assert!(ecx.cursor().next().is_none());
        assert_eq!(ecx.cursor().remaining_path(), "");
        assert!(ecx.cursor().next().is_none());
    }

    #[test]
    fn test_segments_from_root_path() {
        let request = Request::get("/").body(()).unwrap();
        let context = Context::new(request);
        let mut ecx = PreflightContext::new(&context);

        assert_eq!(ecx.cursor().remaining_path(), "");
        assert!(ecx.cursor().next().is_none());
    }
}
