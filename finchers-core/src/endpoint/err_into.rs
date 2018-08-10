#![allow(missing_docs)]

use futures_util::try_future::{self, TryFutureExt};
use std::marker::PhantomData;

use crate::endpoint::{Context, EndpointBase};

#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, U> {
    pub(super) endpoint: E,
    pub(super) _marker: PhantomData<fn() -> U>,
}

impl<E, U> EndpointBase for ErrInto<E, U>
where
    E: EndpointBase,
    E::Error: Into<U>,
{
    type Ok = E::Ok;
    type Error = U;
    type Future = try_future::ErrInto<E::Future, U>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        Some(future.err_into())
    }
}
