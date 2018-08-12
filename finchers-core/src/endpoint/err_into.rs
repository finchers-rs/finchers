use futures_util::try_future::{self, TryFutureExt};
use std::marker::PhantomData;
use std::mem::PinMut;

use endpoint::Endpoint;
use input::{Cursor, Input};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct ErrInto<E, U> {
    pub(super) endpoint: E,
    pub(super) _marker: PhantomData<fn() -> U>,
}

impl<E, U> Endpoint for ErrInto<E, U>
where
    E: Endpoint,
    E::Error: Into<U>,
{
    type Ok = E::Ok;
    type Error = U;
    type Future = try_future::ErrInto<E::Future, U>;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        let (future, cursor) = self.endpoint.apply(input, cursor)?;
        Some((future.err_into(), cursor))
    }
}
