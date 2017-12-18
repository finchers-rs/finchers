use task;
use endpoint::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};


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
    type Task = task::JoinAll<E::Task>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        let tasks: Vec<_> = self.inner
            .iter()
            .map(|e| e.apply(ctx))
            .collect::<Result<_, EndpointError>>()?;
        Ok(task::join_all(tasks))
    }
}
