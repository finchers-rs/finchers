//! Components for constructing asynchronous computations which will be returned from `Endpoint`s.
//!
//! The main trait in this module is `Task`.
//! This trait is an abstraction of asynchronous computations which will be returned from endpoints.
//! The role of this trait is very close to `futures` and hence its design intentionally resembles
//! `Future`. However, some differences are exist for specializing to the purpose of HTTP handling.
//!
//! This trait does not provide any combinators for composing complicate computations.
//! Such combinations are usually performed indirectly by the endpoints or by wrapping the value of
//! `Future`.

use either::Either;
use futures::{Async, Future, IntoFuture};

use crate::error::Error;
use crate::never::Never;
use crate::poll::{Poll, PollResult};

/// Trait representing the asynchronous computation after applying the endpoints.
///
/// See the module level documentation for details.
pub trait Task: Send {
    /// The *inner* type of an output which will be returned from this task.
    type Output;

    /// Perform polling this task and get its result.
    fn poll_task(&mut self) -> PollResult<Self::Output, Error>;
}

impl<L, R> Task for Either<L, R>
where
    L: Task,
    R: Task<Output = L::Output>,
{
    type Output = L::Output;

    #[inline(always)]
    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        match *self {
            Either::Left(ref mut t) => t.poll_task(),
            Either::Right(ref mut t) => t.poll_task(),
        }
    }
}

/// Trait representing the conversion to a `Task`.
pub trait IntoTask {
    /// The type of *output* value.
    type Output;

    /// The type of value to be converted.
    type Task: Task<Output = Self::Output>;

    /// Perform conversion itself into a `Task`.
    fn into_task(self) -> Self::Task;
}

// FIXME: replace the trait bound with `core::ops::Async`
impl<F> IntoTask for F
where
    F: IntoFuture,
    F::Future: Send,
{
    type Output = Result<F::Item, F::Error>;
    type Task = TaskFuture<F::Future>;

    #[inline(always)]
    fn into_task(self) -> Self::Task {
        future(self)
    }
}

/// A helper struct which wraps a `Future` and provides the implementation of `Task`.
#[derive(Debug)]
pub struct TaskFuture<F>(F);

impl<F> From<F> for TaskFuture<F>
where
    F: Future + Send,
{
    fn from(fut: F) -> Self {
        TaskFuture(fut)
    }
}

impl<F> Task for TaskFuture<F>
where
    F: Future + Send,
{
    type Output = Result<F::Item, F::Error>;

    #[inline(always)]
    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        match Future::poll(&mut self.0) {
            Ok(Async::Ready(ready)) => Poll::Ready(Ok(Ok(ready))),
            Ok(Async::NotReady) => Poll::Pending,
            Err(err) => Poll::Ready(Ok(Err(err))),
        }
    }
}

/// Create a task from a `Future`.
pub fn future<F>(future: F) -> TaskFuture<F::Future>
where
    F: IntoFuture,
    F::Future: Send,
{
    TaskFuture::from(IntoFuture::into_future(future))
}

/// A `Task` which will immediately return a value of `T`.
#[derive(Debug)]
pub struct Ready<T>(Option<T>);

impl<T: Send> From<T> for Ready<T> {
    fn from(val: T) -> Self {
        Ready(Some(val))
    }
}

impl<T: Send> Task for Ready<T> {
    type Output = T;

    #[inline(always)]
    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        let val = self.0.take().expect("The task cannot resolve twice");
        Poll::Ready(Ok(val))
    }
}

/// Create a task which will immediately return a value of `T`.
pub fn ready<T: Send>(val: T) -> Ready<T> {
    Ready::from(val)
}

/// A `Task` which will immediately abort with an error value of `E`.
#[derive(Debug)]
pub struct Abort<E> {
    cause: Option<E>,
}

impl<E> From<E> for Abort<E>
where
    E: Into<Error> + Send,
{
    fn from(cause: E) -> Self {
        Abort { cause: Some(cause) }
    }
}

impl<E> Task for Abort<E>
where
    E: Into<Error> + Send,
{
    type Output = Never;

    #[inline(always)]
    fn poll_task(&mut self) -> PollResult<Self::Output, Error> {
        let cause = self.cause.take().expect("The task cannot reject twice");
        Poll::Ready(Err(Into::into(cause)))
    }
}

/// Create a task which will immediately abort the computation with an error value of `E`.
pub fn abort<E>(cause: E) -> Abort<E>
where
    E: Into<Error> + Send,
{
    Abort::from(cause)
}
