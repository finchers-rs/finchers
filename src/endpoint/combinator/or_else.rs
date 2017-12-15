use std::marker::PhantomData;
use std::sync::Arc;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task::{IntoTask, Poll, Task};
use super::chain::Chain;


/// Equivalent to `e.or_else(f)`
pub fn or_else<E, F, R>(endpoint: E, f: F) -> OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoTask<Item = E::Item>,
{
    OrElse {
        endpoint,
        f: Arc::new(f),
        _marker: PhantomData,
    }
}


/// The return type of `or_else()`
#[derive(Debug)]
pub struct OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoTask<Item = E::Item>,
{
    endpoint: E,
    f: Arc<F>,
    _marker: PhantomData<R>,
}

// The implementation of `Endpoint` for `AndThen`.
impl<E, F, R> Endpoint for OrElse<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoTask<Item = E::Item>,
{
    type Item = R::Item;
    type Error = R::Error;
    type Task = OrElseTask<E, F, R>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let fut = self.endpoint.apply(ctx)?;
        Ok(OrElseTask {
            inner: Chain::new(fut, self.f.clone()),
        })
    }
}

#[derive(Debug)]
pub struct OrElseTask<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoTask<Item = E::Item>,
{
    inner: Chain<E::Task, R::Task, Arc<F>>,
}

impl<E, F, R> Task for OrElseTask<E, F, R>
where
    E: Endpoint,
    F: Fn(E::Error) -> R,
    R: IntoTask<Item = E::Item>,
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.inner.poll(ctx, |result, f| match result {
            Ok(item) => Ok(Ok(item)),
            Err(err) => Ok(Err((*f)(err).into_task())),
        })
    }
}
