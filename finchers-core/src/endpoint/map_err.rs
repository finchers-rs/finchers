use futures_util::try_future::{self, TryFutureExt};
use std::mem::PinMut;

use endpoint::Endpoint;
use input::{Cursor, Input};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct MapErr<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E, F, U> Endpoint for MapErr<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Error) -> U + Clone,
{
    type Ok = E::Ok;
    type Error = U;
    type Future = try_future::MapErr<E::Future, F>;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        let (future, cursor) = self.endpoint.apply(input, cursor)?;
        let f = self.f.clone();
        Some((future.map_err(f), cursor))
    }
}
