use futures_util::future;

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

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

impl<'a, T: Clone + 'a> Endpoint<'a> for Value<T> {
    type Output = (T,);
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&self, _: &mut Context<'_>) -> EndpointResult<Self::Future> {
        Ok(future::ready(Ok((self.x.clone(),))))
    }
}
