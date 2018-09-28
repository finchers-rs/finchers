use futures::{Future, Poll};

use endpoint::{ApplyContext, ApplyResult, Endpoint};
use error::Error;

/// Create an endpoint which simply returns a reference to the specified value.
///
/// # Examples
///
/// ```
/// # #[macro_use]
/// # extern crate finchers;
/// # use finchers::prelude::*;
/// # use finchers::endpoint::by_ref;
/// #
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
///     .and(by_ref(conn))
///     .and_then(|id: u32, conn: &Conn| {
///         // ...
/// #       let _ = conn;
/// #       Ok(id)
///     });
/// # drop(endpoint);
/// # }
/// ```
#[inline]
pub fn by_ref<T>(x: T) -> ByRef<T> {
    ByRef { x }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct ByRef<T> {
    x: T,
}

impl<'a, T: 'a> Endpoint<'a> for ByRef<T> {
    type Output = (&'a T,);
    type Future = ByRefFuture<'a, T>;

    fn apply(&'a self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(ByRefFuture { x: &self.x })
    }
}

#[derive(Debug)]
pub struct ByRefFuture<'a, T: 'a> {
    x: &'a T,
}

impl<'a, T: 'a> Future for ByRefFuture<'a, T> {
    type Item = (&'a T,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok((self.x,).into())
    }
}
