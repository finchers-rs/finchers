//! Extensions for constructing Endpoints

pub mod option;
pub mod result;

mod abort;
mod all;
mod and;
mod inspect;
mod just;
mod lazy;
mod left;
mod lift;
mod map;
mod map_async;
mod maybe_done;
mod or;
mod right;

// re-exports
pub use self::abort::{abort, Abort};
pub use self::all::{all, All};
pub use self::and::And;
pub use self::inspect::Inspect;
pub use self::just::{just, Just};
pub use self::lazy::{lazy, Lazy};
pub use self::left::Left;
pub use self::lift::Lift;
pub use self::map::Map;
pub use self::map_async::MapAsync;
pub use self::or::Or;
pub use self::right::Right;

#[doc(inline)]
pub use self::option::EndpointOptionExt;
#[doc(inline)]
pub use self::result::EndpointResultExt;

// ==== EndpointExt ===

use crate::endpoint::{assert_output, Endpoint, IntoEndpoint};
use crate::task::IntoTask;

/// A set of extension methods used for composing complicate endpoints.
pub trait EndpointExt: Endpoint + Sized {
    /// Annotate that the associated type `Output` is equal to `T`.
    #[inline(always)]
    fn as_t<T>(self) -> Self
    where
        Self: Endpoint<Output = T>,
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
        Self::Output: Send,
        E::Output: Send,
    {
        assert_output::<_, (Self::Output, <E::Endpoint as Endpoint>::Output)>(self::and::new(
            self, e,
        ))
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
        E: IntoEndpoint<Output = Self::Output>,
    {
        assert_output::<_, Self::Output>(self::or::new(self, e))
    }

    /// Create an endpoint which returns `None` if the inner endpoint skips the request.
    fn lift(self) -> Lift<Self> {
        assert_output::<_, Option<Self::Output>>(self::lift::new(self))
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U + Clone + Send + Sync,
    {
        assert_output::<_, F::Output>(self::map::new(self, f))
    }

    /// Create an endpoint which do something with the output value from `self`.
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        F: FnOnce(&Self::Output) + Clone + Send + Sync,
    {
        assert_output::<_, Self::Output>(self::inspect::new(self, f))
    }

    /// Create an endpoint which continue an asynchronous computation
    /// from the value returned from `self`.
    fn map_async<F, T>(self, f: F) -> MapAsync<Self, F>
    where
        F: FnOnce(Self::Output) -> T + Clone + Send + Sync,
        T: IntoTask,
        T::Task: Send,
    {
        assert_output::<_, T::Output>(self::map_async::new(self, f))
    }
}

impl<E: Endpoint> EndpointExt for E {}
