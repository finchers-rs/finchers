use futures_core::future::TryFuture;
use futures_util::try_future::{MapOk, TryFutureExt};
use std::mem::PinMut;

use crate::endpoint::{Cursor, Endpoint, EndpointResult};
use crate::error::Error;
use crate::generic::{one, One};
use crate::input::Input;

/// Create an endpoint which executes the provided closure for each request.
///
/// # Examples
///
/// ```
/// # #![feature(rust_2018_preview)]
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
/// #       ready(Ok(conn))
///     });
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

impl<F, R> Endpoint for Lazy<F>
where
    F: Fn(PinMut<'_, Input>) -> R,
    R: TryFuture<Error = Error>,
{
    type Output = One<R::Ok>;
    type Future = MapOk<R, fn(R::Ok) -> Self::Output>;

    fn apply<'c>(
        &self,
        input: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> EndpointResult<'c, Self::Future> {
        Ok(((self.f)(input).map_ok(one as fn(_) -> _), cursor))
    }
}
