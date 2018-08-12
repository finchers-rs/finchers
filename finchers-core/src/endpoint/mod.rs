//! Components for constructing `Endpoint`.

mod and;
mod and_then;
mod err_into;
mod map_err;
mod map_ok;
mod ok;
mod or;
mod or_else;
mod reject;
mod try_chain;

// re-exports
pub use self::and::And;
pub use self::and_then::AndThen;
pub use self::err_into::ErrInto;
pub use self::map_err::MapErr;
pub use self::map_ok::MapOk;
pub use self::ok::{ok, Ok};
pub use self::or::Or;
pub use self::or_else::OrElse;

pub use self::reject::{reject, Reject};

// ====

use std::marker::PhantomData;
use std::mem::PinMut;
use std::rc::Rc;
use std::sync::Arc;

use futures_core::future::TryFuture;

use either::Either;
use generic::{Combine, Func, Tuple};
use input::{Cursor, Input};

/// Trait representing an endpoint.
pub trait Endpoint {
    /// The inner type associated with this endpoint.
    type Ok: Tuple;

    /// The error type.
    type Error;

    /// The type of value which will be returned from `apply`.
    type Future: TryFuture<Ok = Self::Ok, Error = Self::Error>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)>;
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        (*self).apply(input, cursor)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        (**self).apply(input, cursor)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        (**self).apply(input, cursor)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, input: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        (**self).apply(input, cursor)
    }
}

/// Trait representing the transformation into an `Endpoint`.
pub trait IntoEndpoint {
    /// The inner type of associated `Endpoint`.
    type Ok: Tuple;

    /// The error type.
    type Error;

    /// The type of transformed `Endpoint`.
    type Endpoint: Endpoint<Ok = Self::Ok, Error = Self::Error>;

    /// Consume itself and transform into an `Endpoint`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E: Endpoint> IntoEndpoint for E {
    type Ok = E::Ok;
    type Error = E::Error;
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

/// A set of extension methods used for composing complicate endpoints.
pub trait EndpointExt: Endpoint + Sized {
    #[allow(missing_docs)]
    #[inline]
    fn ok<T: Tuple>(self) -> Self
    where
        Self: Endpoint<Ok = T>,
    {
        self
    }

    #[allow(missing_docs)]
    #[inline]
    fn err<E>(self) -> Self
    where
        Self: Endpoint<Error = E>,
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
        E: IntoEndpoint<Ok = Self::Ok>,
    {
        (Or {
            e1: self,
            e2: other.into_endpoint(),
        }).ok::<Self::Ok>()
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

impl<E: Endpoint> EndpointExt for E {}
