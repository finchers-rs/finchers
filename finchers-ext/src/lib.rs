//! Extensions for constructing Endpoints

#![doc(html_root_url = "https://docs.rs/finchers-ext/0.11.0")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(warnings)]

extern crate either;
#[macro_use]
extern crate finchers_core;

pub mod option;
pub mod result;

mod abort;
mod all;
mod and;
mod inspect;
mod just;
mod lazy;
mod left;
mod map;
mod map_async;
mod maybe_done;
mod or;
mod right;

// re-exports
pub use abort::{abort, Abort};
pub use all::{all, All};
pub use and::And;
pub use inspect::Inspect;
pub use just::{just, Just};
pub use lazy::{lazy, Lazy};
pub use left::Left;
pub use map::Map;
pub use map_async::MapAsync;
pub use or::Or;
pub use right::Right;

#[doc(inline)]
pub use option::EndpointOptionExt;
#[doc(inline)]
pub use result::EndpointResultExt;

// ==== EndpointExt ===

use finchers_core::endpoint::{assert_output, Endpoint, IntoEndpoint};
use finchers_core::task::IntoTask;

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
        assert_output::<_, (Self::Output, <E::Endpoint as Endpoint>::Output)>(self::and::new(self, e))
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
