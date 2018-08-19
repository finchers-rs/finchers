use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::{Future, TryFuture};
use pin_utils::unsafe_unpinned;

use crate::endpoint::{Context, Endpoint, EndpointErrorKind, EndpointResult};
use crate::error::Error;

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Fixed<E> {
    pub(super) endpoint: E,
}

impl<E> Endpoint for Fixed<E>
where
    E: Endpoint,
{
    type Output = E::Output;
    type Future = FixedFuture<E::Future>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match self.endpoint.apply(ecx) {
            Ok(future) => Ok(FixedFuture { inner: Ok(future) }),
            Err(err) => {
                while let Some(..) = ecx.next_segment() {}
                Ok(FixedFuture {
                    inner: Err(Some(err)),
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct FixedFuture<F> {
    inner: Result<F, Option<EndpointErrorKind>>,
}

impl<F> FixedFuture<F> {
    unsafe_unpinned!(inner: Result<F, Option<EndpointErrorKind>>);
}

impl<F> Future for FixedFuture<F>
where
    F: TryFuture<Error = Error>,
{
    type Output = Result<F::Ok, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.inner() {
            Ok(ref mut f) => unsafe { PinMut::new_unchecked(f).try_poll(cx) },
            Err(ref mut err) => Poll::Ready(Err(err.take().unwrap().into())),
        }
    }
}
