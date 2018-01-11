#![allow(missing_docs)]

use std::fmt;
use std::sync::Arc;

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
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

pub struct Inspect<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    endpoint: E,
    f: Arc<F>,
}

impl<E, F> Clone for Inspect<E, F>
where
    E: Endpoint + Clone,
    F: Fn(&E::Item),
{
    fn clone(&self) -> Self {
        Inspect {
            endpoint: self.endpoint.clone(),
            f: self.f.clone(),
        }
    }
}

impl<E, F> fmt::Debug for Inspect<E, F>
where
    E: Endpoint + fmt::Debug,
    F: Fn(&E::Item) + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Inspect")
            .field("endpoint", &self.endpoint)
            .field("f", &self.f)
            .finish()
    }
}

impl<E, F> Endpoint for Inspect<E, F>
where
    E: Endpoint,
    F: Fn(&E::Item),
{
    type Item = E::Item;
    type Error = E::Error;
    type Task = task::inspect::Inspect<E::Task, F>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let task = try_opt!(self.endpoint.apply(ctx));
        Some(task::inspect::Inspect {
            task,
            f: self.f.clone(),
        })
    }
}
