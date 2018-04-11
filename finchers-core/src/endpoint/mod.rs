mod context;
mod error;
pub mod task;

use self::task::Future;
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
    type Future: Future<Item = Self::Item> + Send;

    /// Validates the incoming HTTP request,
    /// and returns the instance of `Future` if matched.
    fn apply(&self, cx: &mut Context) -> Option<Self::Future>;
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (*self).apply(cx)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
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
