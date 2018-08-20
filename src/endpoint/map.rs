use futures_core::future::{Future, TryFuture};
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use std::mem::PinMut;
use std::task;
use std::task::Poll;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;
use crate::generic::{one, Func, One, Tuple};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Map<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<'a, E, F> Endpoint<'a> for Map<E, F>
where
    E: Endpoint<'a>,
    F: Func<E::Output> + 'a,
{
    type Output = One<F::Out>;
    type Future = MapFuture<'a, E::Future, F>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let future = self.endpoint.apply(ecx)?;
        Ok(MapFuture {
            future,
            f: Some(&self.f),
        })
    }
}

#[derive(Debug)]
pub struct MapFuture<'a, T, F: 'a> {
    future: T,
    f: Option<&'a F>,
}

impl<'a, T, F> MapFuture<'a, T, F> {
    unsafe_pinned!(future: T);
    unsafe_unpinned!(f: Option<&'a F>);
}

impl<'a, T, F> Future for MapFuture<'a, T, F>
where
    T: TryFuture<Error = Error>,
    T::Ok: Tuple,
    F: Func<T::Ok> + 'a,
{
    type Output = Result<One<F::Out>, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.future().try_poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                let f = self.f().take().expect("this future has already polled.");
                Poll::Ready(result.map(|item| one(f.call(item))))
            }
        }
    }
}
