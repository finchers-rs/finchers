extern crate either;
#[macro_use]
extern crate finchers_core;

pub mod result;

mod abort;
mod all;
mod and;
mod inspect;
mod just;
mod lazy;
mod left;
mod map;
mod maybe_done;
mod or;
mod right;
mod then;
mod try_abort;

// re-exports
pub use abort::{abort, Abort};
pub use all::{all, All};
pub use and::And;
pub use inspect::Inspect;
pub use just::{just, Just};
pub use lazy::{lazy, Lazy};
pub use left::Left;
pub use map::Map;
pub use or::Or;
pub use right::Right;
pub use then::Then;
pub use try_abort::TryAbort;

pub use result::EndpointResultExt;

// ==== EndpointExt ===

use finchers_core::HttpError;
use finchers_core::endpoint::{Endpoint, IntoEndpoint};
use finchers_core::outcome::IntoOutcome;

pub trait EndpointExt: Endpoint + Sized {
    /// Ensure that the associated type `Item` is equal to `T`.
    #[inline(always)]
    fn as_<T>(self) -> Self
    where
        Self: Endpoint<Output = T>,
    {
        self
    }

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
        E: IntoEndpoint,
        Self::Output: Send,
        E::Output: Send,
    {
        assert_endpoint::<_, (Self::Output, <E::Endpoint as Endpoint>::Output)>(self::and::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The future returned from this endpoint is same as the one returned from "self".
    fn left<E>(self, e: E) -> Left<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_endpoint::<_, Self::Output>(self::left::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The future returned from this endpoint is same as the one returned from "e".
    fn right<E>(self, e: E) -> Right<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_endpoint::<_, E::Output>(self::right::new(self, e))
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
        E: IntoEndpoint<Output = Self::Output>,
    {
        assert_endpoint::<_, Self::Output>(self::or::new(self, e))
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U + Clone + Send,
    {
        assert_endpoint::<_, F::Output>(self::map::new(self, f))
    }

    /// Create an endpoint which do something with the output value from "Self".
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        F: FnOnce(&Self::Output) + Clone + Send,
    {
        assert_endpoint::<_, Self::Output>(self::inspect::new(self, f))
    }

    /// Create an endpoint which continue an asynchronous computation
    /// from the value returned from "self".
    ///
    /// The future will abort if the future returned from "f" will be rejected with
    /// an unrecoverable error.
    fn then<F, R>(self, f: F) -> Then<Self, F>
    where
        F: FnOnce(Self::Output) -> R + Clone + Send,
        R: IntoOutcome,
        R::Outcome: Send,
    {
        assert_endpoint::<_, R::Output>(self::then::new(self, f))
    }

    /// Create an endpoint which maps the returned value from "self" to a `Result`.
    ///
    /// The future will abort if the mapped value will be an `Err`.
    fn try_abort<F, T, E>(self, f: F) -> TryAbort<Self, F>
    where
        F: FnOnce(Self::Output) -> Result<T, E> + Clone + Send,
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
    E: Endpoint<Output = T>,
{
    endpoint
}
