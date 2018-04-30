use std::rc::Rc;
use std::sync::Arc;

use super::apply::ApplyRequest;
use super::context::Context;
use input::{Input, RequestBody};
use task::Task;

/// Trait representing an *endpoint*.
pub trait Endpoint {
    /// The inner type associated with this endpoint.
    type Output;

    /// The type of value which will be returned from "Endpoint::apply".
    type Task: Task<Output = Self::Output> + Send;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated task if matched.
    fn apply(&self, cx: &mut Context) -> Option<Self::Task>;

    /// Create an asyncrhonous computation from a request.
    fn apply_request(&self, input: &Input, body: RequestBody) -> ApplyRequest<Self::Task> {
        super::apply::apply_request(self, input, body)
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

impl<E: Endpoint> Endpoint for Rc<E> {
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
