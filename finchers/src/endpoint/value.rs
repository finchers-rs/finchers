use futures::{Future, Poll};

use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;

/// Create an endpoint which simply clones the specified value.
///
/// # Examples
///
/// ```
/// # #[macro_use]
/// # extern crate finchers;
/// # use finchers::prelude::*;
/// # use finchers::endpoint::value;
/// #
/// #[derive(Clone)]
/// struct Conn {
///     // ...
/// #   _p: (),
/// }
///
/// # fn main() {
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
/// #       Ok(conn)
///     });
/// # drop(endpoint);
/// # }
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

impl<T: Clone> Endpoint for Value<T> {
    type Output = (T,);
    type Future = ValueFuture<T>;

    fn apply(&self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(ValueFuture {
            x: Some(self.x.clone()),
        })
    }
}

#[derive(Debug)]
pub struct ValueFuture<T> {
    x: Option<T>,
}

impl<T> Future for ValueFuture<T> {
    type Item = (T,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok((self.x.take().expect("The value has already taken."),).into())
    }
}
