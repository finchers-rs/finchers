use futures_core::future::{Future, TryFuture};
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use crate::endpoint::EndpointBase;
use crate::generic::{Func, Tuple};
use crate::input::{Cursor, Input};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct MapOk<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F> EndpointBase for MapOk<E, F>
where
    E: EndpointBase,
    F: Func<E::Ok> + Clone,
    F::Out: Tuple,
{
    type Ok = F::Out;
    type Error = E::Error;
    type Future = MapOkFuture<E::Future, F>;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        let (future, cursor) = self.endpoint.apply(input, cursor)?;
        let f = self.f.clone();
        Some((MapOkFuture { future, f: Some(f) }, cursor))
    }
}

#[derive(Debug)]
pub struct MapOkFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F> MapOkFuture<T, F> {
    unsafe_pinned!(future: T);
    unsafe_unpinned!(f: Option<F>);
}

impl<T, F> Future for MapOkFuture<T, F>
where
    T: TryFuture,
    T::Ok: Tuple,
    F: Func<T::Ok>,
    F::Out: Tuple,
{
    type Output = Result<F::Out, T::Error>;

    fn poll(mut self: PinMut<Self>, cx: &mut task::Context) -> Poll<Self::Output> {
        match self.future().try_poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                let f = self.f().take().expect("this future has already polled.");
                Poll::Ready(result.map(|item| f.call(item)))
            }
        }
    }
}
