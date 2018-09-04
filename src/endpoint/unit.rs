use std::pin::PinMut;

use futures_core::future::Future;
use futures_core::task;
use futures_core::task::Poll;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

/// Create an endpoint which simply returns an unit (`()`).
pub fn unit() -> Unit {
    Unit { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Unit {
    _priv: (),
}

impl<'a> Endpoint<'a> for Unit {
    type Output = ();
    type Future = UnitFuture;

    #[inline]
    fn apply(&'a self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(UnitFuture { _priv: () })
    }
}

#[derive(Debug)]
pub struct UnitFuture {
    _priv: (),
}

impl Future for UnitFuture {
    type Output = Result<(), Error>;

    #[inline]
    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Ok(()))
    }
}
