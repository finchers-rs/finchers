//! Components for constructing `EndpointBase`.

mod context;
pub mod ext;

pub use self::context::{Context, EncodedStr, Segment, Segments};
use crate::future::Future;
use crate::generic::Tuple;
use crate::output::Responder;
use hyper::body::Payload;
use std::rc::Rc;
use std::sync::Arc;

#[inline(always)]
crate fn assert_output<E, T: Tuple>(endpoint: E) -> E
where
    E: EndpointBase<Output = T>,
{
    endpoint
}

/// Trait representing an endpoint.
pub trait EndpointBase {
    /// The inner type associated with this endpoint.
    type Output: Tuple;

    /// The type of value which will be returned from `apply`.
    type Future: Future<Output = Self::Output>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&self, cx: &mut Context) -> Option<Self::Future>;
}

impl<'a, E: EndpointBase> EndpointBase for &'a E {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (*self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Box<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Rc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Arc<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
    }
}

#[allow(missing_docs)]
pub trait Endpoint: Send + Sync + 'static + sealed::Sealed {
    type Output: Responder<Body = Self::Body>;
    type Body: Payload;
    type Future: Future<Output = Self::Output> + Send + 'static;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future>;
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
    E::Future: Send + 'static,
{
    type Output = E::Output;
    type Body = <E::Output as Responder>::Body;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        EndpointBase::apply(self, cx)
    }
}

/// Trait representing the transformation into an `EndpointBase`.
pub trait IntoEndpoint {
    /// The inner type of associated `EndpointBase`.
    type Output: Tuple;

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
