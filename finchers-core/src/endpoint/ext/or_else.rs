#![allow(missing_docs)]

use futures_core::future::TryFuture;
use futures_util::try_future::{self, TryFutureExt};

use crate::endpoint::{Context, EndpointBase};

#[derive(Debug, Copy, Clone)]
pub struct OrElse<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, R> EndpointBase for OrElse<E, F>
where
    E: EndpointBase,
    F: FnOnce(E::Error) -> R + Clone,
    R: TryFuture<Ok = E::Ok>,
{
    type Ok = R::Ok;
    type Error = R::Error;
    type Future = try_future::OrElse<E::Future, R, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let f1 = self.endpoint.apply(cx)?;
        let f = self.f.clone();
        Some(f1.or_else(f))
    }
}
