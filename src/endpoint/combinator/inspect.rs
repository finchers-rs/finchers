#![allow(missing_docs)]

use std::sync::Arc;
use futures::{Future, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};


pub fn inspect<E, F>(endpoint: E, f: F) -> Inspect<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    Inspect {
        endpoint,
        f: Arc::new(f),
    }
}


#[derive(Debug)]
pub struct Inspect<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    endpoint: E,
    f: Arc<F>,
}

impl<E, F> Endpoint for Inspect<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    type Item = E::Item;
    type Error = E::Error;
    type Future = InspectFuture<E, F>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(InspectFuture {
            inner,
            f: self.f.clone(),
        })
    }
}


#[derive(Debug)]
pub struct InspectFuture<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    inner: E::Future,
    f: Arc<F>,
}

impl<E, F> Future for InspectFuture<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    type Item = E::Item;
    type Error = E::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.inner.poll());
        (*self.f)(&item);
        Ok(item.into())
    }
}
