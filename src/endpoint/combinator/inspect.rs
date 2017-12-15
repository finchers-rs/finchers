#![allow(missing_docs)]

use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task;


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
    type Task = task::Inspect<E::Task, F>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(task::inspect(inner, self.f.clone()))
    }
}
