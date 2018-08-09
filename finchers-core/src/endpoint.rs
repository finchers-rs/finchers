//! Components for constructing `EndpointBase`.

mod context;
pub mod ext;

use futures_core::future::TryFuture;
use std::rc::Rc;
use std::sync::Arc;

pub use self::context::{Context, EncodedStr, Segment, Segments};
pub use self::ext::EndpointExt;

use crate::error::Error;
use crate::generic::Tuple;
use crate::output::Responder;

/// Trait representing an endpoint.
pub trait EndpointBase {
    /// The inner type associated with this endpoint.
    type Ok: Tuple;

    /// The error type.
    type Error;

    /// The type of value which will be returned from `apply`.
    type Future: TryFuture<Ok = Self::Ok, Error = Self::Error>;

    /// Perform checking the incoming HTTP request and returns
    /// an instance of the associated Future if matched.
    fn apply(&self, cx: &mut Context) -> Option<Self::Future>;
}

impl<'a, E: EndpointBase> EndpointBase for &'a E {
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (*self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Box<E> {
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Rc<E> {
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
    }
}

impl<E: EndpointBase> EndpointBase for Arc<E> {
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (**self).apply(cx)
    }
}

#[allow(missing_docs)]
pub trait Endpoint: Send + Sync + 'static + sealed::Sealed {
    type Ok: Responder;
    type Error: Into<Error>;
    type Future: TryFuture<Ok = Self::Ok, Error = Self::Error> + Send + 'static;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future>;
}

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl<E> Sealed for E
    where
        E: EndpointBase,
        E::Ok: Responder,
        E::Error: Into<Error>,
    {}
}

impl<E> Endpoint for E
where
    E: EndpointBase + Send + Sync + 'static,
    E::Ok: Responder,
    E::Error: Into<Error>,
    E::Future: Send + 'static,
{
    type Ok = E::Ok;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        EndpointBase::apply(self, cx)
    }
}

/// Trait representing the transformation into an `EndpointBase`.
pub trait IntoEndpoint {
    /// The inner type of associated `EndpointBase`.
    type Ok: Tuple;

    /// The error type.
    type Error;

    /// The type of transformed `EndpointBase`.
    type Endpoint: EndpointBase<Ok = Self::Ok, Error = Self::Error>;

    /// Consume itself and transform into an `EndpointBase`.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<E: EndpointBase> IntoEndpoint for E {
    type Ok = E::Ok;
    type Error = E::Error;
    type Endpoint = E;

    #[inline]
    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}
