extern crate either;
#[macro_use]
extern crate finchers_core;

pub mod option;
pub mod result;

mod abort;
mod all;
mod and;
mod common;
mod inspect;
mod just;
mod lazy;
mod left;
mod map;
mod maybe_done;
mod or;
mod right;
mod then;

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

#[doc(inline)]
pub use option::EndpointOptionExt;
#[doc(inline)]
pub use result::EndpointResultExt;

// ==== EndpointExt ===

use common::assert_output;
use finchers_core::endpoint::{Endpoint, IntoEndpoint};
use finchers_core::outcome::IntoOutcome;

pub trait EndpointExt: Endpoint + Sized {
    /// Enforce that the associated type `Output` is equal to `T`.
    #[inline(always)]
    fn as_t<T>(self) -> Self
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
    fn and<E>(self, e: E) -> And<Self, E::Endpoint>
    where
        E: IntoEndpoint,
        Self::Output: Send,
        E::Output: Send,
    {
        assert_output::<_, (Self::Output, <E::Endpoint as Endpoint>::Output)>(self::and::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The future returned from this endpoint is same as the one returned from "self".
    fn left<E>(self, e: E) -> Left<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_output::<_, Self::Output>(self::left::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The future returned from this endpoint is same as the one returned from "e".
    fn right<E>(self, e: E) -> Right<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_output::<_, E::Output>(self::right::new(self, e))
    }

    /// Create an endpoint which evaluates "self" and "e" sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either "self" or "e" matched "better" to the input.
    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        E: IntoEndpoint<Output = Self::Output>,
    {
        assert_output::<_, Self::Output>(self::or::new(self, e))
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U + Clone + Send,
    {
        assert_output::<_, F::Output>(self::map::new(self, f))
    }

    /// Create an endpoint which do something with the output value from "Self".
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        F: FnOnce(&Self::Output) + Clone + Send,
    {
        assert_output::<_, Self::Output>(self::inspect::new(self, f))
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
        assert_output::<_, R::Output>(self::then::new(self, f))
    }
}

impl<E: Endpoint> EndpointExt for E {}
