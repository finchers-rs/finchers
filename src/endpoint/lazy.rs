#![allow(deprecated)]
#![doc(hidden)]
#![deprecated(since = "0.12.0-alpha.4", note = "use `lazy2()` instead.")]

use std::pin::PinMut;

use futures_core::future::TryFuture;
use futures_util::try_future::{MapOk, TryFutureExt};

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;
use crate::input::Input;

pub fn lazy<F, R>(f: F) -> Lazy<F>
where
    F: Fn(PinMut<'_, Input>) -> R,
    R: TryFuture<Error = Error>,
{
    Lazy { f }
}

#[derive(Debug, Copy, Clone)]
pub struct Lazy<F> {
    f: F,
}

impl<'a, F, R> Endpoint<'a> for Lazy<F>
where
    F: Fn(PinMut<'_, Input>) -> R + 'a,
    R: TryFuture<Error = Error> + 'a,
{
    type Output = (R::Ok,);
    #[allow(clippy::type_complexity)]
    type Future = MapOk<R, fn(R::Ok) -> Self::Output>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok((self.f)(PinMut::new(ecx.input())).map_ok((|ok| (ok,)) as fn(_) -> _))
    }
}
