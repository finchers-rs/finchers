use super::*;
use Context;
use finchers_core::{Error, Input};
use futures::Future;
use std::rc::Rc;
use std::sync::Arc;

/// Abstruction of an endpoint.
pub trait Endpoint {
    /// The type *on success*.
    type Item;

    /// The type of returned value from `apply`.
    type Future: Future<Item = Self::Item, Error = Error>;

    /// Validates the incoming HTTP request,
    /// and returns the instance of `Task` if matched.
    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future>;
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        (*self).apply(input, ctx)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        (**self).apply(input, ctx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        (**self).apply(input, ctx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        (**self).apply(input, ctx)
    }
}

/// Abstruction of types to be convert to an `Endpoint`.
pub trait IntoEndpoint {
    /// The return type
    type Item;
    /// The type of value returned from `into_endpoint`.
    type Endpoint: Endpoint<Item = Self::Item>;

    /// Convert itself into `Self::Endpoint`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E: Endpoint> IntoEndpoint for E {
    type Item = E::Item;
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

impl IntoEndpoint for () {
    type Item = ();
    type Endpoint = EndpointOk<()>;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        ok(())
    }
}

impl<E: IntoEndpoint> IntoEndpoint for Vec<E> {
    type Item = Vec<E::Item>;
    type Endpoint = JoinAll<E::Endpoint>;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        join_all(self)
    }
}

/// A shortcut of `IntoEndpoint::into_endpoint()`
#[inline]
pub fn endpoint<E: IntoEndpoint>(endpoint: E) -> E::Endpoint {
    endpoint.into_endpoint()
}
