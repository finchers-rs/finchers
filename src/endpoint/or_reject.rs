use std::pin::PinMut;

use futures_core::future::{Future, TryFuture};
use futures_core::task;
use futures_core::task::Poll;
use pin_utils::unsafe_unpinned;

use crate::endpoint::{Context, Endpoint, EndpointError, EndpointResult};
use crate::error::Error;

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct OrReject<E> {
    pub(super) endpoint: E,
}

impl<'a, E: Endpoint<'a>> Endpoint<'a> for OrReject<E> {
    type Output = E::Output;
    type Future = OrRejectFuture<E::Future>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match self.endpoint.apply(ecx) {
            Ok(future) => Ok(OrRejectFuture { inner: Ok(future) }),
            Err(err) => {
                while let Some(..) = ecx.next_segment() {}
                Ok(OrRejectFuture {
                    inner: Err(Some(err.into())),
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct OrRejectFuture<F> {
    inner: Result<F, Option<Error>>,
}

impl<F> OrRejectFuture<F> {
    unsafe_unpinned!(inner: Result<F, Option<Error>>);
}

impl<F> Future for OrRejectFuture<F>
where
    F: TryFuture<Error = Error>,
{
    type Output = Result<F::Ok, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.inner() {
            Ok(ref mut f) => unsafe { PinMut::new_unchecked(f).try_poll(cx) },
            Err(ref mut err) => Poll::Ready(Err(err.take().unwrap())),
        }
    }
}

// ==== OrRejectWith ====

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct OrRejectWith<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<'a, E, F, R> Endpoint<'a> for OrRejectWith<E, F>
where
    E: Endpoint<'a>,
    F: Fn(EndpointError, &mut Context<'_>) -> R + 'a,
    R: Into<Error> + 'a,
{
    type Output = E::Output;
    type Future = OrRejectFuture<E::Future>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match self.endpoint.apply(ecx) {
            Ok(future) => Ok(OrRejectFuture { inner: Ok(future) }),
            Err(err) => {
                while let Some(..) = ecx.next_segment() {}
                let err = (self.f)(err, ecx).into();
                Ok(OrRejectFuture {
                    inner: Err(Some(err)),
                })
            }
        }
    }
}
