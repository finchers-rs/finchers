#![allow(deprecated)]

use std::marker::PhantomData;
use std::pin::PinMut;

use futures_util::future;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;
use crate::input::Input;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.3",
    note = "This endpoint is going to remove before releasing 0.12.0."
)]
pub fn reject<F, E>(f: F) -> Reject<F, E>
where
    F: Fn(PinMut<'_, Input>) -> E,
    E: Into<Error>,
{
    (Reject {
        f,
        _marker: PhantomData,
    }).with_output::<()>()
}

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.3",
    note = "This endpoint is going to remove before releasing 0.12.0."
)]
#[derive(Debug)]
pub struct Reject<F, E> {
    f: F,
    _marker: PhantomData<fn() -> E>,
}

impl<F: Copy, E> Copy for Reject<F, E> {}

impl<F: Clone, E> Clone for Reject<F, E> {
    fn clone(&self) -> Self {
        Reject {
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'a, F, E> Endpoint<'a> for Reject<F, E>
where
    F: Fn(PinMut<'_, Input>) -> E + 'a,
    E: Into<Error> + 'a,
{
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        while let Some(..) = ecx.next_segment() {}
        Ok(future::ready(Err((self.f)(ecx.input()).into())))
    }
}
