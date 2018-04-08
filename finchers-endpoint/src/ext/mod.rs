mod abort;
mod abort_with;
mod and;
mod chain;
mod left;
mod map;
mod or;
mod right;
mod then;
mod try_abort;

pub use self::abort::Abort;
pub use self::abort_with::AbortWith;
pub use self::and::And;
pub use self::left::Left;
pub use self::map::Map;
pub use self::or::Or;
pub use self::right::Right;
pub use self::then::Then;
pub use self::try_abort::TryAbort;

use endpoint::{Endpoint, IntoEndpoint};
use finchers_core::Caller;
use finchers_core::error::HttpError;
use futures::IntoFuture;

pub trait EndpointExt: Endpoint + Sized {
    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The returned future from this endpoint contains both futures from
    /// "self" and "e" and resolved as a pair of values returned from theirs.
    fn and<E>(self, e: E) -> And<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_endpoint::<_, (Self::Item, <E::Endpoint as Endpoint>::Item)>(self::and::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either "self" or "e" matched "better" to the input.
    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        E: IntoEndpoint<Item = Self::Item>,
    {
        assert_endpoint::<_, Self::Item>(self::or::new(self, e))
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

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        F: Caller<Self::Item> + Clone,
    {
        assert_endpoint::<_, F::Output>(self::map::new(self, f))
    }

    /// Create an endpoint which continue an asynchronous computation
    /// from the value returned from "self".
    ///
    /// The returned future from "f" always success and never returns an error.
    /// If "f" will reject with an unrecoverable error, use "try_abort" instead.
    fn then<F>(self, f: F) -> Then<Self, F>
    where
        F: Caller<Self::Item> + Clone,
        F::Output: IntoFuture<Error = !>,
    {
        assert_endpoint::<_, <F::Output as IntoFuture>::Item>(self::then::new(self, f))
    }

    /// Create an endpoint which always abort with the returned value from "self".
    fn abort(self) -> Abort<Self>
    where
        Self::Item: HttpError,
    {
        assert_endpoint::<_, !>(self::abort::new(self))
    }

    /// Create an endpoint which always abort with mapping the value returned from "self"
    /// to an error value.
    fn abort_with<F>(self, f: F) -> AbortWith<Self, F>
    where
        F: Caller<Self::Item> + Clone,
        F::Output: HttpError,
    {
        assert_endpoint::<_, !>(self::abort_with::new(self, f))
    }

    /// Create an endpoint which continue an asynchronous computation
    /// from the value returned from "self".
    ///
    /// The future will abort if the future returned from "f" will be rejected to
    /// an unrecoverable error.
    fn try_abort<F>(self, f: F) -> TryAbort<Self, F>
    where
        F: Caller<Self::Item> + Clone,
        F::Output: IntoFuture,
        <F::Output as IntoFuture>::Error: HttpError,
    {
        assert_endpoint::<_, <F::Output as IntoFuture>::Item>(self::try_abort::new(self, f))
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
