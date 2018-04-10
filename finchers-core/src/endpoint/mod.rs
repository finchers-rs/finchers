mod context;
mod error;

use Input;
use futures::Future;
use std::rc::Rc;
use std::sync::Arc;

// re-exports
pub use self::context::{Context, Segment, Segments};
pub use self::error::{Error, ErrorKind};

/// Abstruction of an endpoint.
pub trait Endpoint {
    /// The *internal* type of this endpoint.
    type Item;

    /// The type of future returned from `apply`.
    type Future: Future<Item = Self::Item, Error = Error>;

    /// Validates the incoming HTTP request,
    /// and returns the instance of `Future` if matched.
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
