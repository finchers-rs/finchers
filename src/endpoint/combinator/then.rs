use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task::{IntoTask, Poll, Task};
use super::chain::Chain;


/// Equivalent to `e.then(f)`
pub fn then<E, F, R>(endpoint: E, f: F) -> Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoTask,
{
    Then {
        endpoint,
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `then()`
#[derive(Debug)]
pub struct Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoTask,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

impl<E, F, R> Endpoint for Then<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoTask,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = ThenTask<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let fut = self.endpoint.apply(ctx)?;
        Ok(ThenTask {
            inner: Chain::new(fut, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct ThenTask<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoTask,
{
    inner: Chain<E::Task, R::Task, Arc<F>>,
}

impl<E, F, R> Task for ThenTask<E, F, R>
where
    E: Endpoint,
    F: Fn(Result<E::Item, E::Error>) -> R,
    R: IntoTask,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner
            .poll(ctx, |result, f| Ok(Err((*f)(result).into_task())))
    }
}
