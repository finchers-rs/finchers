//! Extensions for constructing Endpoints

mod and;
mod and_then;
mod err;
mod err_into;
mod map_err;
mod map_ok;
mod maybe_done;
mod ok;
mod or;
mod or_else;

// re-exports
pub use self::and::And;
pub use self::and_then::AndThen;
pub use self::err::{err, Err};
pub use self::err_into::ErrInto;
pub use self::map_err::MapErr;
pub use self::map_ok::MapOk;
pub use self::ok::{ok, Ok};
pub use self::or::Or;
pub use self::or_else::OrElse;

// ==== EndpointExt ===

use crate::either::Either;
use crate::endpoint::{EndpointBase, IntoEndpoint};
use crate::future::TryFuture;
use crate::generic::{Combine, Func, One, Tuple};
use std::marker::PhantomData;

/// A set of extension methods used for composing complicate endpoints.
pub trait EndpointExt: EndpointBase + Sized {
    #[allow(missing_docs)]
    #[inline]
    fn ok<T: Tuple>(self) -> Self
    where
        Self: EndpointBase<Ok = T>,
    {
        self
    }

    #[allow(missing_docs)]
    #[inline]
    fn err<E>(self) -> Self
    where
        Self: EndpointBase<Error = E>,
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
        Self::Ok: Combine<E::Ok>,
    {
        (And {
            e1: self,
            e2: other.into_endpoint(),
        }).ok::<<Self::Ok as Combine<E::Ok>>::Out>()
        .err::<Either<Self::Error, E::Error>>()
    }

    /// Create an endpoint which evaluates `self` and `e` sequentially.
    ///
    /// The returned future from this endpoint contains the one returned
    /// from either `self` or `e` matched "better" to the input.
    fn or<E>(self, other: E) -> Or<Self, E::Endpoint>
    where
        E: IntoEndpoint,
    {
        (Or {
            e1: self,
            e2: other.into_endpoint(),
        }).ok::<One<Either<Self::Ok, E::Ok>>>()
        .err::<Either<Self::Error, E::Error>>()
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map_ok<F>(self, f: F) -> MapOk<Self, F>
    where
        F: Func<Self::Ok> + Clone,
        F::Out: Tuple,
    {
        (MapOk { endpoint: self, f })
            .ok::<F::Out>()
            .err::<Self::Error>()
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn map_err<F, U>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> U + Clone,
    {
        (MapErr { endpoint: self, f }).ok::<Self::Ok>().err::<U>()
    }

    /// Create an endpoint which maps the returned value to a different type.
    fn err_into<U>(self) -> ErrInto<Self, U>
    where
        Self::Error: Into<U>,
    {
        (ErrInto {
            endpoint: self,
            _marker: PhantomData,
        }).ok::<Self::Ok>()
        .err::<U>()
    }

    #[allow(missing_docs)]
    fn and_then<F>(self, f: F) -> AndThen<Self, F>
    where
        F: Func<Self::Ok> + Clone,
        F::Out: TryFuture<Error = Self::Error>,
        <F::Out as TryFuture>::Ok: Tuple,
    {
        (AndThen { endpoint: self, f })
            .ok::<<F::Out as TryFuture>::Ok>()
            .err::<Self::Error>()
    }

    #[allow(missing_docs)]
    fn or_else<F, R>(self, f: F) -> OrElse<Self, F>
    where
        F: FnOnce(Self::Error) -> R + Clone,
        R: TryFuture<Ok = Self::Ok>,
    {
        (OrElse { endpoint: self, f })
            .ok::<Self::Ok>()
            .err::<R::Error>()
    }
}

impl<E: EndpointBase> EndpointExt for E {}
