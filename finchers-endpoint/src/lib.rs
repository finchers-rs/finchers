extern crate finchers_core;
#[macro_use]
extern crate futures;
extern crate either;
extern crate http;

mod all;
mod and;
mod and_then;
mod inspect;
mod left;
mod map;
mod maybe_done;
mod ok;
mod or;
mod right;
mod try_abort;

// re-exports
pub use all::{all, All};
pub use and::And;
pub use and_then::AndThen;
pub use inspect::Inspect;
pub use left::Left;
pub use map::Map;
pub use ok::{ok, Ok};
pub use or::Or;
pub use right::Right;
pub use try_abort::TryAbort;

use finchers_core::HttpError;
use finchers_core::endpoint::{Endpoint, IntoEndpoint};
use futures::IntoFuture;

pub trait EndpointExt: Endpoint + Sized {
    /// Create an endpoint which evaluates "self" and "e" sequentially
    /// and then returns their results as a pair.
    ///
    /// The returned future from this endpoint contains both futures from
    /// "self" and "e" and resolved as a pair of values returned from theirs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let e1 = ok("foo");
    /// let e2 = ok("bar");
    /// let endpoint = e1.and(e2);
    /// ```
    fn and<E>(self, e: E) -> And<Self, E::Endpoint>
    where
        Self::Item: Send,
        E: IntoEndpoint,
        E::Item: Send,
    {
        assert_endpoint::<_, (Self::Item, <E::Endpoint as Endpoint>::Item)>(self::and::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The future returned from this endpoint is same as the one returned from "self".
    fn left<E>(self, e: E) -> Left<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_endpoint::<_, Self::Item>(self::left::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The future returned from this endpoint is same as the one returned from "e".
    fn right<E>(self, e: E) -> Right<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_endpoint::<_, E::Item>(self::right::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either "self" or "e" matched "better" to the input.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let e1 = path("/foo/bar").map(|_| "path 1");
    /// let e2 = path("/foo/baz").map(|_| "path 2");
    /// let endpoint = e1.or(e2);
    /// ```
    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        E: IntoEndpoint<Item = Self::Item>,
    {
        assert_endpoint::<_, Self::Item>(self::or::new(self, e))
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Item) -> U + Clone + Send,
    {
        assert_endpoint::<_, F::Output>(self::map::new(self, f))
    }

    /// Create an endpoint which do something with the output value from "Self".
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        F: FnOnce(&Self::Item) + Clone + Send,
    {
        assert_endpoint::<_, Self::Item>(self::inspect::new(self, f))
    }

    /// Create an endpoint which continue an asynchronous computation
    /// from the value returned from "self".
    ///
    /// The future will abort if the future returned from "f" will be rejected with
    /// an unrecoverable error.
    fn and_then<F, R>(self, f: F) -> AndThen<Self, F>
    where
        F: FnOnce(Self::Item) -> R + Clone + Send,
        R: IntoFuture,
        R::Future: Send,
        R::Error: HttpError,
    {
        assert_endpoint::<_, R::Item>(self::and_then::new(self, f))
    }

    /// Create an endpoint which maps the returned value from "self" to a `Result`.
    ///
    /// The future will abort if the mapped value will be an `Err`.
    fn try_abort<F, T, E>(self, f: F) -> TryAbort<Self, F>
    where
        F: FnOnce(Self::Item) -> Result<T, E> + Clone + Send,
        E: HttpError,
    {
        // FIXME: replace the trait bound with `Try`
        assert_endpoint::<_, T>(self::try_abort::new(self, f))
    }
}

impl<E: Endpoint> EndpointExt for E {}

#[inline]
fn assert_endpoint<E, T>(endpoint: E) -> E
where
    E: Endpoint<Item = T>,
{
    endpoint
}
