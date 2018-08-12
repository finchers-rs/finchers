use std::mem::PinMut;

use futures_core::future::TryFuture;
use futures_util::try_future::{self, TryFutureExt};

use endpoint::EndpointBase;
use input::{Cursor, Input};

#[allow(missing_docs)]
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

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        let (future, cursor) = self.endpoint.apply(input, cursor)?;
        let f = self.f.clone();
        Some((future.or_else(f), cursor))
    }
}
