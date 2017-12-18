#![allow(missing_docs)]

use std::sync::Arc;

use endpoint::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};
use task;


pub fn inspect<E, F, A, B>(endpoint: E, f: F) -> Inspect<E::Endpoint, F>
where
    E: IntoEndpoint<A, B>,
    F: Fn(&A),
{
    Inspect {
        endpoint: endpoint.into_endpoint(),
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
    type Task = task::Inspect<E::Task, fn(&E::Item), F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        let inner = self.endpoint.apply(ctx)?;
        Ok(task::shared::inspect(inner, self.f.clone()))
    }
}
