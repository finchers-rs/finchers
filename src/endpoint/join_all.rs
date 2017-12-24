#![allow(missing_docs)]

use task;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};


pub fn join_all<I, E, A, B>(iter: I) -> JoinAll<E::Endpoint>
where
    I: IntoIterator<Item = E>,
    E: IntoEndpoint<A, B>,
{
    JoinAll {
        inner: iter.into_iter().map(IntoEndpoint::into_endpoint).collect(),
    }
}


#[derive(Debug)]
pub struct JoinAll<E: Endpoint> {
    inner: Vec<E>,
}

impl<E: Endpoint> Endpoint for JoinAll<E> {
    type Item = Vec<E::Item>;
    type Error = E::Error;
    type Task = task::join_all::JoinAll<E::Task>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let inner: Vec<E::Task> = self.inner
            .iter()
            .map(|e| e.apply(ctx))
            .collect::<Option<_>>()?;
        Some(task::join_all::JoinAll { inner })
    }
}
