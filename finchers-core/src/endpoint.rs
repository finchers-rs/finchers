//! Components for constructing `EndpointBase`.

mod context;

pub use self::context::{Context, EncodedStr, Segment, Segments};
use crate::output::Responder;
use crate::task::Task;
use std::rc::Rc;
use std::sync::Arc;

#[inline(always)]
crate fn assert_output<E, T>(endpoint: E) -> E
where
    E: EndpointBase<Output = T>,
{
    endpoint
}

/// Trait representing an endpoint.
pub trait EndpointBase {
    /// The inner type associated with this endpoint.
    type Output;

    /// The type of value which will be returned from `apply`.
    type Task: Task<Output = Self::Output>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated task if matched.
    fn apply(&self, cx: &mut Context) -> Option<Self::Task>;
}

impl<'a, E: EndpointBase> EndpointBase for &'a E {
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (*self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Box<E> {
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (**self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Rc<E> {
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (**self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Arc<E> {
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        (**self).apply(cx)
    }
}

#[allow(missing_docs)]
pub trait Endpoint: Send + Sync + 'static + sealed::Sealed {
    type Output: Responder;
    type Task: Task<Output = Self::Output> + Send + 'static;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task>;
}

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl<E> Sealed for E
    where
        E: EndpointBase,
        E::Output: Responder,
    {
    }
}

impl<E> Endpoint for E
where
    E: EndpointBase + Send + Sync + 'static,
    E::Output: Responder,
    E::Task: Send + 'static,
{
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        EndpointBase::apply(self, cx)
    }
}

/// Trait representing the transformation into an `EndpointBase`.
pub trait IntoEndpoint {
    /// The inner type of associated `EndpointBase`.
    type Output;

    /// The type of transformed `EndpointBase`.
    type Endpoint: EndpointBase<Output = Self::Output>;

    /// Consume itself and transform into an `EndpointBase`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E: EndpointBase> IntoEndpoint for E {
    type Output = E::Output;
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}
