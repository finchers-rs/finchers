#![allow(missing_docs)]

use futures::{Future, Poll};
use hyper;
use tokio_core::reactor::Handle;
use tokio_service::Service;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use response::Responder;
use task::Task;


/// A wrapper of a `NewEndpoint`, to provide hyper's HTTP services
#[derive(Debug, Clone)]
pub struct EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    endpoint: E,
}

impl<E> EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    pub fn new(endpoint: E, _handle: &Handle) -> Self {
        // TODO: clone the instance of Handle and implement it to Context
        EndpointService { endpoint }
    }
}

impl<E> Service for EndpointService<E>
where
    E: Endpoint,
    E::Item: Responder,
    E::Error: Responder,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = RespondFuture<TaskFuture<E::Task>>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let ctx = Context::from_hyper(req);
        let task = create_task_future(&self.endpoint, ctx);
        RespondFuture::new(task)
    }
}


pub(crate) fn create_task_future<E: Endpoint>(
    endpoint: &E,
    mut ctx: Context,
) -> Result<TaskFuture<E::Task>, EndpointError> {
    let mut result = endpoint.apply(&mut ctx);

    // check if the remaining path segments are exist.
    if ctx.count_remaining_segments() > 0 {
        result = Err(EndpointError::Skipped);
    }

    result.map(|task| TaskFuture::new(task, ctx))
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct TaskFuture<T: Task> {
    task: T,
    ctx: Context,
}

impl<T: Task> TaskFuture<T> {
    #[allow(missing_docs)]
    pub fn new(task: T, ctx: Context) -> Self {
        TaskFuture { task, ctx }
    }
}

impl<T: Task> Future for TaskFuture<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.task.poll(&mut self.ctx)
    }
}


/// The type of a future returned from `EndpointService::call()`
#[derive(Debug)]
pub struct RespondFuture<F: Future>
where
    F::Item: Responder,
    F::Error: Responder,
{
    inner: Result<F, Option<EndpointError>>,
}

impl<F: Future> RespondFuture<F>
where
    F::Item: Responder,
    F::Error: Responder,
{
    #[allow(missing_docs)]
    pub fn new(result: Result<F, EndpointError>) -> Self {
        RespondFuture {
            inner: result.map_err(Some),
        }
    }
}

impl<F: Future> Future for RespondFuture<F>
where
    F::Item: Responder,
    F::Error: Responder,
{
    type Item = hyper::Response;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Check the result of `Endpoint::apply()`.
        let inner = match self.inner.as_mut() {
            Ok(inner) => inner,
            Err(err) => {
                let err = err.take().expect("cannot reject twice");
                return Ok(err.into_response().into());
            }
        };

        // Query the future returned from the endpoint
        let item = inner.poll();
        // ...and convert its success/error value to `hyper::Response`.
        let item = item.map(|item| item.map(Responder::into_response))
            .map_err(Responder::into_response);

        Ok(item.unwrap_or_else(Into::into))
    }
}
