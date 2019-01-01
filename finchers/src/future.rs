#![allow(missing_docs)]

use {
    crate::{
        endpoint::Cursor, //
        error::Error,
        input::Input,
    },
    futures::Future,
    std::{
        cell::Cell, //
        marker::PhantomData,
        ptr::NonNull,
        rc::Rc,
    },
};

pub use futures::{try_ready, Async, Poll};

pub trait EndpointFuture {
    type Output;

    fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error>;
}

impl<F> EndpointFuture for F
where
    F: Future,
    F::Error: Into<Error>,
{
    type Output = F::Item;

    fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error> {
        with_set_cx(cx, || self.poll()).map_err(Into::into)
    }
}

pub fn poll_fn<T, E>(
    f: impl FnMut(&mut Context<'_>) -> Poll<T, E>,
) -> impl EndpointFuture<Output = T>
where
    E: Into<Error>,
{
    #[allow(missing_debug_implementations)]
    struct PollFn<F>(F);

    impl<F, T, E> EndpointFuture for PollFn<F>
    where
        F: FnMut(&mut Context<'_>) -> Poll<T, E>,
        E: Into<Error>,
    {
        type Output = T;

        fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error> {
            (self.0)(cx).map_err(Into::into)
        }
    }

    PollFn(f)
}

/// The contexual information per request during polling the future returned from endpoints.
///
/// The value of this context can be indirectly access by calling `with_get_cx()`.
#[derive(Debug)]
pub struct Context<'a> {
    input: &'a mut Input,
    cursor: &'a Cursor,
    _marker: PhantomData<Rc<()>>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(input: &'a mut Input, cursor: &'a Cursor) -> Context<'a> {
        Context {
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

impl<'a> std::ops::Deref for Context<'a> {
    type Target = Input;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.input
    }
}

impl<'a> std::ops::DerefMut for Context<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.input()
    }
}

thread_local!(static CX: Cell<Option<NonNull<Context<'static>>>> = Cell::new(None));

struct SetOnDrop(Option<NonNull<Context<'static>>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        CX.with(|cx| cx.set(self.0));
    }
}

#[allow(clippy::cast_ptr_alignment)]
fn with_set_cx<R>(current: &mut Context<'_>, f: impl FnOnce() -> R) -> R {
    CX.with(|cx| {
        cx.set(Some(unsafe {
            NonNull::new_unchecked(current as *mut Context<'_> as *mut () as *mut Context<'static>)
        }))
    });
    let _reset = SetOnDrop(None);
    f()
}

/// Acquires a mutable reference to `Context` from the current task context
/// and executes the provided function using its value.
///
/// This function is usually used to access the value of `Input` within the `Future`
/// returned by the `Endpoint`.
///
/// # Panics
///
/// A panic will occur if you call this function inside the provided closure `f`, since the
/// reference to `Context` on the task context is invalidated while executing `f`.
#[deprecated]
pub fn with_get_cx<R>(f: impl FnOnce(&mut Context<'_>) -> R) -> R {
    let prev = CX.with(|cx| cx.replace(None));
    let _reset = SetOnDrop(prev);
    match prev {
        Some(mut ptr) => unsafe { f(ptr.as_mut()) },
        None => panic!("The reference to Context is not set at the current context."),
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled."]
pub enum MaybeDone<F: EndpointFuture> {
    Ready(F::Output),
    Pending(F),
    Gone,
}

impl<F: EndpointFuture> MaybeDone<F> {
    pub fn take_item(&mut self) -> Option<F::Output> {
        match std::mem::replace(self, MaybeDone::Gone) {
            MaybeDone::Ready(output) => Some(output),
            _ => None,
        }
    }
}

impl<F: EndpointFuture> EndpointFuture for MaybeDone<F> {
    type Output = ();

    fn poll_endpoint(&mut self, cx: &mut Context<'_>) -> Poll<Self::Output, Error> {
        let polled = match self {
            MaybeDone::Ready(..) => return Ok(Async::Ready(())),
            MaybeDone::Pending(ref mut future) => future.poll_endpoint(cx)?,
            MaybeDone::Gone => panic!("This future has already polled"),
        };
        match polled {
            Async::Ready(output) => {
                *self = MaybeDone::Ready(output);
                Ok(Async::Ready(()))
            }
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

#[derive(Debug)]
pub enum TryChain<F1, F2, T> {
    First(F1, Option<T>),
    Second(F2),
    Empty,
}

#[allow(missing_debug_implementations)]
pub enum TryChainAction<F2>
where
    F2: EndpointFuture,
{
    Future(F2),
    Output(Result<F2::Output, Error>),
}

impl<F1, F2, T> TryChain<F1, F2, T>
where
    F1: EndpointFuture,
    F2: EndpointFuture,
{
    pub(super) fn new(f1: F1, data: T) -> TryChain<F1, F2, T> {
        TryChain::First(f1, Some(data))
    }

    pub(super) fn try_poll<F>(&mut self, cx: &mut Context<'_>, f: F) -> Poll<F2::Output, Error>
    where
        F: FnOnce(Result<F1::Output, Error>, T) -> TryChainAction<F2>,
    {
        let mut f = Some(f);
        loop {
            let (out, data) = match self {
                TryChain::First(ref mut f1, ref mut data) => match f1.poll_endpoint(cx) {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(ok)) => (Ok(ok), data.take().unwrap()),
                    Err(err) => (Err(err), data.take().unwrap()),
                },
                TryChain::Second(ref mut f2) => return f2.poll_endpoint(cx),
                TryChain::Empty => panic!("This future has already polled."),
            };

            let f = f.take().unwrap();
            match f(out, data) {
                TryChainAction::Future(f2) => {
                    *self = TryChain::Second(f2);
                    continue;
                }
                TryChainAction::Output(out) => {
                    *self = TryChain::Empty;
                    return out.map(Async::Ready);
                }
            }
        }
    }
}
