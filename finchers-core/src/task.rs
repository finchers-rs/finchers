use either::Either;
use error::Error;
use futures::Future;
use input::{Input, RequestBody};

pub use futures::Async;

/// A type alias for values returned from "Task::poll_task".
pub type PollTask<T> = Result<Async<T>, Error>;

/// The context during polling a task.
pub struct Context<'a> {
    input: &'a Input,
    body: &'a mut Option<RequestBody>,
    // FIXME: add `futures::task::Context`
}

impl<'a> Context<'a> {
    pub fn new(input: &'a Input, body: &'a mut Option<RequestBody>) -> Context<'a> {
        Context { input, body }
    }

    pub fn input(&self) -> &Input {
        self.input
    }

    pub fn body(&mut self) -> Option<RequestBody> {
        self.body.take()
    }
}

/// Trait representing the asynchronous computation after applying the endpoints.
///
/// This trait provides a *basic* abstraction for asynchronous computation.
/// It is intentionally designed to be close to "futures::Future", but the following differences
/// are exist:
///
/// * It can take only the one associated type, for representing an *output* type of the task.
///   The error type is always fixed to "Error".
/// * It does not provide any combinators for composing complicate computations.
///   Such combinations are usually performed indirectly by the endpoints.
/// * It will take an argument which enables to access the context during the computation
///   (similar to "futures2", but more specialized to the purpose of HTTP handling).
pub trait Task {
    /// The *inner* type of an output which will be returned from this task.
    type Output;

    /// Perform polling this task and get its result.
    fn poll_task(&mut self, cx: &mut Context) -> PollTask<Self::Output>;
}

impl<L, R> Task for Either<L, R>
where
    L: Task,
    R: Task<Output = L::Output>,
{
    type Output = L::Output;

    fn poll_task(&mut self, cx: &mut Context) -> PollTask<Self::Output> {
        match *self {
            Either::Left(ref mut t) => t.poll_task(cx),
            Either::Right(ref mut t) => t.poll_task(cx),
        }
    }
}

#[derive(Debug)]
pub struct CompatTask<F>(F);

impl<F> From<F> for CompatTask<F>
where
    F: Future,
    F::Error: Into<Error>,
{
    fn from(fut: F) -> CompatTask<F> {
        CompatTask(fut)
    }
}

impl<F> Task for CompatTask<F>
where
    F: Future,
    F::Error: Into<Error>,
{
    type Output = F::Item;

    fn poll_task(&mut self, _: &mut Context) -> PollTask<Self::Output> {
        self.0.poll().map_err(Into::into)
    }
}
