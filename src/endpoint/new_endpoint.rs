use std::rc::Rc;
use std::sync::Arc;
use futures::Future;
use tokio_core::reactor::Handle;

use super::endpoint::Endpoint;


/// A factory of `Endpoint`
pub trait NewEndpoint {
    /// The return type of `Endpoint`
    type Item;

    /// The error type of `Endpoint`
    type Error;

    /// The future type of `Endpoint`
    type Future: Future<Item = Self::Item, Error = Self::Error>;

    /// The type of `Endpoint` returned from `new_endpoint()`
    type Endpoint: Endpoint<Item = Self::Item, Error = Self::Error, Future = Self::Future>;

    /// Create a new instance of `Endpoint` with given event loop
    fn new_endpoint(&self, handle: &Handle) -> Self::Endpoint;
}

impl<F, E> NewEndpoint for F
where
    F: Fn(&Handle) -> E,
    E: Endpoint,
{
    type Item = E::Item;
    type Error = E::Error;
    type Future = E::Future;
    type Endpoint = E;

    fn new_endpoint(&self, handle: &Handle) -> Self::Endpoint {
        (*self)(handle)
    }
}

impl<E: NewEndpoint> NewEndpoint for Rc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Future = E::Future;
    type Endpoint = E::Endpoint;

    fn new_endpoint(&self, handle: &Handle) -> Self::Endpoint {
        (**self).new_endpoint(handle)
    }
}

impl<E: NewEndpoint> NewEndpoint for Arc<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Future = E::Future;
    type Endpoint = E::Endpoint;

    fn new_endpoint(&self, handle: &Handle) -> Self::Endpoint {
        (**self).new_endpoint(handle)
    }
}
