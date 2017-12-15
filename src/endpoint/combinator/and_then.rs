use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task::{IntoTask, Poll, Task};
use super::chain::Chain;


/// Equivalent to `e.and_then(f)`
pub fn and_then<E, F, R>(endpoint: E, f: F) -> AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoTask<Error = E::Error>,
{
    AndThen {
        endpoint,
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `and_then()`
#[derive(Debug)]
pub struct AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoTask<Error = E::Error>,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Endpoint for AndThen<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoTask<Error = E::Error>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = AndThenTask<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let f = self.endpoint.apply(ctx)?;
        Ok(AndThenTask {
            inner: Chain::new(f, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct AndThenTask<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoTask<Error = E::Error>,
{
    inner: Chain<E::Task, R::Task, Arc<F>>,
}

impl<E, F, R> Task for AndThenTask<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
    R: IntoTask<Error = E::Error>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(ctx, |result, f| match result {
            Ok(item) => Ok(Err((*f)(item).into_task())),
            Err(err) => Err(err),
        })
    }
}
