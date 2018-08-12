use futures_core::future::{Future, TryFuture};
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use endpoint::Endpoint;
use error::Error;
use generic::{Func, Tuple};
use input::{Cursor, Input};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Map<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: Func<E::Output> + Clone,
    F::Out: Tuple,
{
    type Output = F::Out;
    type Future = MapFuture<E::Future, F>;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        let (future, cursor) = self.endpoint.apply(input, cursor)?;
        let f = self.f.clone();
        Some((MapFuture { future, f: Some(f) }, cursor))
    }
}

#[derive(Debug)]
pub struct MapFuture<T, F> {
    future: T,
    f: Option<F>,
}

impl<T, F> MapFuture<T, F> {
    unsafe_pinned!(future: T);
    unsafe_unpinned!(f: Option<F>);
}

impl<T, F> Future for MapFuture<T, F>
where
    T: TryFuture<Error = Error>,
    T::Ok: Tuple,
    F: Func<T::Ok>,
    F::Out: Tuple,
{
    type Output = Result<F::Out, Error>;

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
