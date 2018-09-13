use std::pin::PinMut;

use futures_core::future::Future;
use futures_core::task;
use futures_core::task::Poll;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

/// Create an endpoint which simply clones the specified value.
///
/// # Examples
///
/// ```
/// # extern crate finchers;
/// # extern crate futures_util;
/// # use finchers::prelude::*;
/// # use finchers::endpoint::value;
/// # use finchers::path;
/// # use futures_util::future::ready;
/// #
/// #[derive(Clone)]
/// struct Conn {
///     // ...
/// #   _p: (),
/// }
///
/// let conn = {
///     // do some stuff...
/// #   Conn { _p: () }
/// };
///
/// let endpoint = path!(@get / "posts" / u32 /)
///     .and(value(conn))
///     .and_then(|id: u32, conn: Conn| {
///         // ...
/// #       drop(id);
/// #       ready(Ok(conn))
///     });
/// # drop(endpoint);
/// ```
#[inline]
pub fn value<T: Clone>(x: T) -> Value<T> {
    (Value { x }).with_output::<(T,)>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Value<T> {
    x: T,
}

impl<'a, T: Clone + 'a> Endpoint<'a> for Value<T> {
    type Output = (T,);
    type Future = ValueFuture<'a, T>;

    fn apply(&'a self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(ValueFuture { x: &self.x })
    }
}

#[derive(Debug)]
pub struct ValueFuture<'a, T: 'a> {
    x: &'a T,
}

impl<'a, T: Clone + 'a> Future for ValueFuture<'a, T> {
    type Output = Result<(T,), Error>;

    fn poll(self: PinMut<'_, Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Ok((self.x.clone(),)))
    }
}
