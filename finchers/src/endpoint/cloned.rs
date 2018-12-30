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
/// # use finchers::endpoint::cloned;
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
///     .and(cloned(conn))
///     .and_then(|id: u32, conn: Conn| {
///         // ...
/// #       drop(id);
/// #       Ok(conn)
///     });
/// # drop(endpoint);
/// # }
/// ```
#[inline]
pub fn cloned<T: Clone>(x: T) -> Cloned<T> {
    (Cloned { x }).with_output::<(T,)>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Cloned<T> {
    x: T,
}

impl<'a, T: Clone + 'a> Endpoint<'a> for Cloned<T> {
    type Output = (T,);
    type Future = ClonedFuture<T>;

    fn apply(&'a self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(ClonedFuture {
            x: Some(self.x.clone()),
        })
    }
}

#[derive(Debug)]
pub struct ClonedFuture<T> {
    x: Option<T>,
}

impl<T> Future for ClonedFuture<T> {
    type Item = (T,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok((self.x.take().expect("The value has already taken."),).into())
    }
}
