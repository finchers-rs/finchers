use super::apply::ApplyRequest;
use super::context::Context;
use crate::input::Input;
use crate::task::Task;
use std::sync::Arc;

/// Trait representing an endpoint.
pub trait Endpoint: Send + Sync {
    /// The inner type associated with this endpoint.
    type Output;

    /// The type of value which will be returned from `apply`.
    type Task: Task<Output = Self::Output>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated task if matched.
    fn apply(&self, cx: &mut Context) -> Option<Self::Task>;

    /// Create an asyncrhonous computation from a request.
    fn apply_request(&self, input: &Input) -> ApplyRequest<Self::Task> {
        super::apply::apply_request(self, input)
    }
}

impl<'a, E: Endpoint> Endpoint for &'a E {
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (*self).apply(cx)
    }
}

impl<E: Endpoint> Endpoint for Box<E> {
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (**self).apply(cx)
    }
}

impl<E: Endpoint> Endpoint for Arc<E> {
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (**self).apply(cx)
    }
}

/// Trait representing the transformation into an `Endpoint`.
pub trait IntoEndpoint {
    /// The inner type of associated `Endpoint`.
    type Output;

    /// The type of transformed `Endpoint`.
    type Endpoint: Endpoint<Output = Self::Output>;

    /// Consume itself and transform into an `Endpoint`.
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
