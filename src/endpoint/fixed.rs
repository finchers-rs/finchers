use std::mem::PinMut;
use std::task;
use std::task::Poll;

use futures_core::future::{Future, TryFuture};
use pin_utils::unsafe_unpinned;

use crate::endpoint::Endpoint;
use crate::error::{no_route, Error};
use crate::input::{Cursor, Input};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Fixed<E> {
    pub(super) endpoint: E,
}

impl<E> Endpoint for Fixed<E>
where
    E: Endpoint,
{
    type Output = E::Output;
    type Future = FixedFuture<E::Future>;

    fn apply<'c>(
        &self,
        input: PinMut<'_, Input>,
        mut cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        match self.endpoint.apply(input, cursor.clone()) {
            Some((future, cursor)) => Some((
                FixedFuture {
                    inner: Some(future),
                },
                cursor,
            )),
            None => {
                let _ = cursor.by_ref().count();
                Some((FixedFuture { inner: None }, cursor))
            }
        }
    }
}

#[derive(Debug)]
pub struct FixedFuture<F> {
    inner: Option<F>,
}

impl<F> FixedFuture<F> {
    unsafe_unpinned!(inner: Option<F>);
}

impl<F> Future for FixedFuture<F>
where
    F: TryFuture<Error = Error>,
{
    type Output = Result<F::Ok, Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match self.inner() {
            Some(ref mut f) => unsafe { PinMut::new_unchecked(f).try_poll(cx) },
            None => Poll::Ready(Err(no_route())),
        }
    }
}
