use either::Either;
use error::Error;
use futures::{Async, Future, IntoFuture};
use input::{Input, RequestBody};

/// A type alias for values returned from "Outcome::poll_task".
#[derive(Debug)]
pub enum PollOutcome<T> {
    Ready(T),
    Abort(Error),
    Pending,
}

impl<T, E> From<Result<Async<T>, E>> for PollOutcome<T>
where
    E: Into<Error>,
{
    fn from(val: Result<Async<T>, E>) -> Self {
        match val {
            Ok(Async::Ready(val)) => PollOutcome::Ready(val),
            Ok(Async::NotReady) => PollOutcome::Pending,
            Err(e) => PollOutcome::Abort(Into::into(e)),
        }
    }
}

#[macro_export]
macro_rules! try_poll_outcome {
    ($e:expr) => {
        match PollOutcome::from($e) {
            PollOutcome::Ready(v) => v,
            PollOutcome::Abort(e) => return PollOutcome::Abort(e),
            PollOutcome::Pending => return PollOutcome::Pending,
        }
    };
}

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
pub trait Outcome {
    /// The *inner* type of an output which will be returned from this task.
    type Output;

    /// Perform polling this task and get its result.
    fn poll_outcome(&mut self, cx: &mut Context) -> PollOutcome<Self::Output>;
}

impl<L, R> Outcome for Either<L, R>
where
    L: Outcome,
    R: Outcome<Output = L::Output>,
{
    type Output = L::Output;

    fn poll_outcome(&mut self, cx: &mut Context) -> PollOutcome<Self::Output> {
        match *self {
            Either::Left(ref mut t) => t.poll_outcome(cx),
            Either::Right(ref mut t) => t.poll_outcome(cx),
        }
    }
}

#[derive(Debug)]
pub struct CompatOutcome<F>(F);

impl<F> From<F> for CompatOutcome<F>
where
    F: Future,
    F::Error: Into<Error>,
{
    fn from(fut: F) -> Self {
        CompatOutcome(fut)
    }
}

impl<F> Outcome for CompatOutcome<F>
where
    F: Future,
    F::Error: Into<Error>,
{
    type Output = F::Item;

    fn poll_outcome(&mut self, _: &mut Context) -> PollOutcome<Self::Output> {
        Future::poll(&mut self.0).into()
    }
}

pub trait IntoOutcome {
    type Output;
    type Outcome: Outcome<Output = Self::Output>;

    fn into_outcome(self) -> Self::Outcome;
}

impl<F> IntoOutcome for F
where
    F: IntoFuture,
    F::Error: Into<Error>,
{
    type Output = F::Item;
    type Outcome = CompatOutcome<F::Future>;

    fn into_outcome(self) -> Self::Outcome {
        CompatOutcome::from(IntoFuture::into_future(self))
    }
}
