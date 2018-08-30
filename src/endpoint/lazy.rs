use futures_core::future::TryFuture;
use futures_util::try_future::{MapOk, TryFutureExt};
use std::mem::PinMut;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;
use crate::input::Input;

/// Create an endpoint which executes the provided closure for each request.
///
/// # Examples
///
/// ```
/// # #![feature(futures_api)]
/// #
/// # extern crate finchers;
/// # extern crate futures_core;
/// # extern crate futures_util;
/// # extern crate failure;
/// # use finchers::endpoint::{lazy, EndpointExt};
/// # use finchers::route;
/// # use futures_core::future::Future;
/// # use futures_util::future::ready;
/// # use futures_util::try_future::TryFutureExt;
/// use failure::Fallible;
///
/// # struct Conn { _p: (), }
/// #
/// #[derive(Default)]
/// struct ConnPool {
///     // ...
/// #   _p: (),
/// }
///
/// impl ConnPool {
///     fn get_conn(&self) -> impl Future<Output = Fallible<Conn>> {
///         // ...
/// #       ready(Ok(Conn { _p: () }))
///     }
/// }
///
/// let pool = ConnPool::default();
/// let acquire_conn = lazy(move |_| {
///     pool.get_conn()
///         .map_err(Into::into)
/// });
///
/// let endpoint = route!(@get / "posts" / u32 /)
///     .and(acquire_conn)
///     .and_then(|id: u32, conn: Conn| {
///         // ...
/// #       drop(id);
/// #       ready(Ok(conn))
///     });
/// # drop(endpoint);
/// ```
pub fn lazy<F, R>(f: F) -> Lazy<F>
where
    F: Fn(PinMut<'_, Input>) -> R,
    R: TryFuture<Error = Error>,
{
    Lazy { f }
}

#[allow(missing_docs)]
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
    #[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
    type Future = MapOk<R, fn(R::Ok) -> Self::Output>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok((self.f)(ecx.input()).map_ok((|ok| (ok,)) as fn(_) -> _))
    }
}
