use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;

/// Create an endpoint which simply returns an unit (`()`).
#[inline]
pub fn unit() -> Unit {
    (Unit { _priv: () }).with_output::<()>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Unit {
    _priv: (),
}

impl Endpoint for Unit {
    type Output = ();
    type Future = UnitFuture;

    #[inline]
    fn apply(&self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(UnitFuture { _priv: () })
    }
}

#[derive(Debug)]
pub struct UnitFuture {
    _priv: (),
}

impl ::futures::Future for UnitFuture {
    type Item = ();
    type Error = Error;

    #[inline]
    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        Ok(().into())
    }
}
