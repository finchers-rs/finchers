#![allow(missing_docs)]

use std::rc::Rc;
use std::sync::Arc;
use futures::Future;
use tokio_core::reactor::Handle;

use super::endpoint::Endpoint;


pub trait NewEndpoint {
    type Item;
    type Error;
    type Future: Future<Item = Self::Item, Error = Self::Error>;
    type Endpoint: Endpoint<Item = Self::Item, Error = Self::Error, Future = Self::Future>;

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
