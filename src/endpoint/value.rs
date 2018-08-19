use futures_util::future;
use std::mem::PinMut;

use crate::endpoint::{Endpoint, EndpointResult};
use crate::error::Error;
use crate::generic::{one, One};
use crate::input::{Cursor, Input};

/// Create an endpoint which simply clones the specified value.
///
/// # Examples
///
/// ```
/// # #![feature(rust_2018_preview)]
/// # extern crate finchers;
/// # extern crate futures_util;
/// # use finchers::endpoint::{value, EndpointExt};
/// # use finchers::route;
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
/// let endpoint = route!(@get / "posts" / u32 /)
///     .and(value(conn))
///     .and_then(|id: u32, conn: Conn| {
///         // ...
/// #       ready(Ok(conn))
///     });
/// ```
pub fn value<T: Clone>(x: T) -> Value<T> {
    Value { x }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Value<T> {
    x: T,
}

impl<T: Clone> Endpoint for Value<T> {
    type Output = One<T>;
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(
        &self,
        _: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> EndpointResult<'c, Self::Future> {
        Ok((future::ready(Ok(one(self.x.clone()))), cursor))
    }
}
