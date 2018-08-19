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

impl<E, F> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: Func<E::Output> + Clone,
{
    type Output = One<F::Out>;
    type Future = MapFuture<E::Future, F>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let future = self.endpoint.apply(ecx)?;
        let f = self.f.clone();
        Ok(MapFuture { future, f: Some(f) })
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
