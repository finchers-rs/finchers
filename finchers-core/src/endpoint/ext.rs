//! Extensions for constructing Endpoints

pub mod option;
pub mod result;

mod and;
mod just;
mod lift;
mod map;
mod maybe_done;
mod or;
mod then;

// re-exports
pub use self::and::And;
pub use self::just::{just, Just};
pub use self::lift::Lift;
pub use self::map::Map;
pub use self::or::Or;
pub use self::then::Then;

#[doc(inline)]
pub use self::option::EndpointOptionExt;
#[doc(inline)]
pub use self::result::EndpointResultExt;

// ==== EndpointExt ===

use crate::either::Either;
use crate::endpoint::{assert_output, EndpointBase, IntoEndpoint};
use crate::future::Future;
use crate::generic::{Combine, Func, One, Tuple};

/// A set of extension methods used for composing complicate endpoints.
pub trait EndpointExt: EndpointBase + Sized {
    /// Annotate that the associated type `Output` is equal to `T`.
    #[inline(always)]
    fn as_t<T>(self) -> Self
    where
        Self: EndpointBase<Output = One<T>>,
    {
        self
    }

    /// Create an endpoint which evaluates `self` and `e` and returns a pair of their tasks.
    ///
    /// The returned future from this endpoint contains both futures from
    /// `self` and `e` and resolved as a pair of values returned from theirs.
    fn and<E>(self, other: E) -> And<Self, E::Endpoint>
    where
        E: IntoEndpoint,
        Self::Output: Combine<E::Output>,
    {
        assert_output::<_, <Self::Output as Combine<E::Output>>::Out>(And {
            e1: self,
            e2: other.into_endpoint(),
        })
    }

    /// Create an endpoint which evaluates `self` and `e` sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either `self` or `e` matched "better" to the input.
    fn or<E>(self, other: E) -> Or<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        assert_output::<_, One<Either<Self::Output, E::Output>>>(Or {
            e1: self,
            e2: other.into_endpoint(),
        })
    }

    /// Create an endpoint which returns `None` if the inner endpoint skips the request.
    fn lift(self) -> Lift<Self> {
        assert_output::<_, One<Option<Self::Output>>>(Lift { endpoint: self })
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        F: Func<Self::Output> + Clone,
        F::Out: Tuple,
    {
        assert_output::<_, F::Out>(Map { endpoint: self, f })
    }

    /// Create an endpoint which continue an asynchronous computation
    /// from the value returned from `self`.
    fn then<F>(self, f: F) -> Then<Self, F>
    where
        F: Func<Self::Output> + Clone,
        F::Out: Future,
        <F::Out as Future>::Output: Tuple,
    {
        assert_output::<_, <F::Out as Future>::Output>(Then { endpoint: self, f })
    }
}

impl<E: EndpointBase> EndpointExt for E {}
