mod context;

use outcome::Outcome;
use std::rc::Rc;
use std::sync::Arc;

// re-exports
pub use self::context::{Context, Segment, Segments};

/// Trait representing an *endpoint*.
pub trait Endpoint {
    /// The inner type associated with this endpoint.
    type Output;

    /// The type of asynchronous "Outcome" which will be returned from "apply".
    type Outcome: Outcome<Output = Self::Output> + Send;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated task if matched.
    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome>;
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Output = E::Output;
    type Outcome = E::Outcome;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        (*self).apply(cx)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Output = E::Output;
    type Outcome = E::Outcome;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        (**self).apply(cx)
    }
}

impl<E: Endpoint> Endpoint for Rc<E> {
    type Output = E::Output;
    type Outcome = E::Outcome;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        (**self).apply(cx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Output = E::Output;
    type Outcome = E::Outcome;

    fn apply(&self, cx: &mut Context) -> Option<Self::Outcome> {
        (**self).apply(cx)
    }
}

/// Abstraction of types to be convert to an `Endpoint`.
pub trait IntoEndpoint {
    /// The return type
    type Output;

    /// The type of value returned from `into_endpoint`.
    type Endpoint: Endpoint<Output = Self::Output>;

    /// Convert itself into `Self::Endpoint`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E: Endpoint> IntoEndpoint for E {
    type Output = E::Output;
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}
