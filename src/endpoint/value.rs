use endpoint::{Context, Endpoint, EndpointResult};
use error::Error;

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

impl<'a, T: Clone + 'a> ::futures::Future for ValueFuture<'a, T> {
    type Item = (T,);
    type Error = Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        Ok((self.x.clone(),).into())
    }
}
