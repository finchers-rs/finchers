#![allow(missing_docs)]

use task::{self, TaskResult};
use super::super::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};

pub fn skip_all<I, E, A, B>(iter: I) -> SkipAll<E::Endpoint>
where
    I: IntoIterator<Item = E>,
    E: IntoEndpoint<A, B>,
{
    SkipAll {
        endpoints: iter.into_iter().map(|e| e.into_endpoint()).collect(),
    }
}

#[derive(Debug)]
pub struct SkipAll<E: Endpoint> {
    endpoints: Vec<E>,
}

impl<E: Endpoint> Endpoint for SkipAll<E> {
    type Item = ();
    type Error = E::Error;
    type Task = TaskResult<(), E::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        for endpoint in &self.endpoints {
            let _ = endpoint.apply(ctx)?;
        }
        Ok(task::ok(()))
    }
}
