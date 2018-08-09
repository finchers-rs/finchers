//! Extensions for constructing Endpoints

pub mod option;
pub mod result;

mod all;
mod and;
mod inspect;
mod just;
mod lazy;
mod left;
mod lift;
mod map;
mod maybe_done;
mod or;
mod right;
mod then;

// re-exports
pub use self::all::{all, All};
pub use self::and::And;
pub use self::inspect::Inspect;
pub use self::just::{just, Just};
pub use self::lazy::{lazy, Lazy};
pub use self::left::Left;
pub use self::lift::Lift;
pub use self::map::Map;
pub use self::or::Or;
pub use self::right::Right;
pub use self::then::Then;

#[doc(inline)]
pub use self::option::EndpointOptionExt;
#[doc(inline)]
pub use self::result::EndpointResultExt;

// ==== EndpointExt ===

use crate::either::Either;
use crate::endpoint::{assert_output, EndpointBase, IntoEndpoint};
use crate::future::Future;
use crate::generic::{Combine, Tuple};

/// A set of extension methods used for composing complicate endpoints.
pub trait EndpointExt: EndpointBase + Sized {
    /// Annotate that the associated type `Output` is equal to `T`.
    #[inline(always)]
    fn as_t<T>(self) -> Self
    where
        Self: EndpointBase<Output = T>,
    {
        self
    }

    /// Create an endpoint which evaluates `self` and `e` and returns a pair of their tasks.
    ///
    /// The returned future from this endpoint contains both futures from
    /// `self` and `e` and resolved as a pair of values returned from theirs.
    fn and<E>(self, e: E) -> And<Self, E::Endpoint>
    where
        E: IntoEndpoint,
        Self::Output: Tuple + Combine<E::Output>,
        E::Output: Tuple,
    {
        assert_output::<_, <Self::Output as Combine<E::Output>>::Out>(self::and::new(self, e))
    }

    /// Create an endpoint which evaluates `self` and `e` and returns the task of `self` if matched.
    fn left<E>(self, e: E) -> Left<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_output::<_, Self::Output>(self::left::new(self, e))
    }

    /// Create an endpoint which evaluates `self` and `e` and returns the task of `e` if matched.
    fn right<E>(self, e: E) -> Right<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_output::<_, E::Output>(self::right::new(self, e))
    }

    /// Create an endpoint which evaluates `self` and `e` sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either `self` or `e` matched "better" to the input.
    fn or<E>(self, e: E) -> Or<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_output::<_, Either<Self::Output, E::Output>>(self::or::new(self, e))
    }

    /// Create an endpoint which returns `None` if the inner endpoint skips the request.
    fn lift(self) -> Lift<Self> {
        assert_output::<_, Option<Self::Output>>(self::lift::new(self))
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U + Clone,
    {
        assert_output::<_, F::Output>(self::map::new(self, f))
    }

    /// Create an endpoint which do something with the output value from `self`.
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        F: FnOnce(&Self::Output) + Clone,
    {
        assert_output::<_, Self::Output>(self::inspect::new(self, f))
    }

    /// Create an endpoint which continue an asynchronous computation
    /// from the value returned from `self`.
    fn then<F, T>(self, f: F) -> Then<Self, F>
    where
        F: FnOnce(Self::Output) -> T + Clone,
        T: Future,
    {
        assert_output::<_, T::Output>(self::then::new(self, f))
    }
}

impl<E: EndpointBase> EndpointExt for E {}
