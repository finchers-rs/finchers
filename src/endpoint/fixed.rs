#![allow(deprecated)]

use std::pin::PinMut;

use futures_core::future::{Future, TryFuture};
use futures_core::task;
use futures_core::task::Poll;
use pin_utils::unsafe_unpinned;

use crate::endpoint::{Context, Endpoint, EndpointError, EndpointResult};
use crate::error::Error;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.3",
    note = "This struct is going to remove before releasing 0.12.0."
)]
#[derive(Debug, Copy, Clone)]
pub struct Fixed<E> {
    pub(super) endpoint: E,
}

impl<'a, E: Endpoint<'a>> Endpoint<'a> for Fixed<E> {
    type Output = E::Output;
    type Future = FixedFuture<E::Future>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
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
    inner: Result<F, Option<EndpointError>>,
}

impl<F> FixedFuture<F> {
    unsafe_unpinned!(inner: Result<F, Option<EndpointError>>);
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
