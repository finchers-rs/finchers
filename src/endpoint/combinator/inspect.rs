#![allow(missing_docs)]

use futures::{future, Future};

use context::Context;
use endpoint::{Endpoint, EndpointResult};


pub fn inspect<E, F>(endpoint: E, f: F) -> Inspect<E, F> {
    Inspect { endpoint, f }
}


#[derive(Debug)]
pub struct Inspect<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F> Endpoint for Inspect<E, F>
where
    E: Endpoint,
    F: FnOnce(&E::Item),
{
    type Item = E::Item;
    type Error = E::Error;
    type Future = future::Inspect<E::Future, F>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let f = self.endpoint.apply(ctx)?;
        Ok(f.inspect(self.f))
    }
}
