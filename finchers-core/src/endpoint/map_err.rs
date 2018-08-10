#![allow(missing_docs)]

use futures_util::try_future::{self, TryFutureExt};

use crate::endpoint::{Context, EndpointBase};

#[derive(Debug, Copy, Clone)]
pub struct MapErr<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, U> EndpointBase for MapErr<E, F>
where
    E: EndpointBase,
    F: FnOnce(E::Error) -> U + Clone,
{
    type Ok = E::Ok;
    type Error = U;
    type Future = try_future::MapErr<E::Future, F>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        let f = self.f.clone();
        Some(future.map_err(f))
    }
}
