use futures::{Future, Poll};

use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;

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
pub struct ByRefFuture<'a, T> {
    x: &'a T,
}

impl<'a, T: 'a> Future for ByRefFuture<'a, T> {
    type Item = (&'a T,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok((self.x,).into())
    }
}

#[cfg(test)]
mod tests {
    use crate::endpoint;
    use crate::prelude::*;
    use crate::server;

    #[test]
    #[ignore]
    fn compiletest_by_ref() {
        let endpoint = endpoint::syntax::verb::get()
            .and(endpoint::syntax::param::<u32>())
            .and(endpoint::by_ref(String::from("Hello, world")))
            .and(endpoints::body::text())
            .and_then(|id: u32, s: &String, body: String| {
                Ok(format!("id={}, s={}, body={}", id, s, body))
            });

        server::start(endpoint).serve("127.0.0.1:4000").unwrap();
    }
}
