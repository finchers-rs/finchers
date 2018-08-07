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

use crate::poll::Poll;
use either::Either;
use futures::{Async, Future, IntoFuture};

/// Trait representing the asynchronous computation after applying the endpoints.
///
/// See the module level documentation for details.
pub trait Task {
    /// The *inner* type of an output which will be returned from this task.
    type Output;

    /// Perform polling this task and get its result.
    fn poll_task(&mut self) -> Poll<Self::Output>;
}

impl<L, R> Task for Either<L, R>
where
    L: Task,
    R: Task,
{
    type Output = Either<L::Output, R::Output>;

    #[inline(always)]
    fn poll_task(&mut self) -> Poll<Self::Output> {
        match *self {
            Either::Left(ref mut t) => t.poll_task().map(Either::Left),
            Either::Right(ref mut t) => t.poll_task().map(Either::Right),
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
impl<F: IntoFuture> IntoTask for F {
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

impl<F: Future> From<F> for TaskFuture<F> {
    fn from(fut: F) -> Self {
        TaskFuture(fut)
    }
}

impl<F: Future> Task for TaskFuture<F> {
    type Output = Result<F::Item, F::Error>;

    #[inline(always)]
    fn poll_task(&mut self) -> Poll<Self::Output> {
        match Future::poll(&mut self.0) {
            Ok(Async::Ready(ready)) => Poll::Ready(Ok(ready)),
            Ok(Async::NotReady) => Poll::Pending,
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

/// Create a task from a `Future`.
pub fn future<F: IntoFuture>(future: F) -> TaskFuture<F::Future> {
    TaskFuture::from(IntoFuture::into_future(future))
}

/// A `Task` which will immediately return a value of `T`.
#[derive(Debug)]
pub struct Ready<T>(Option<T>);

impl<T> From<T> for Ready<T> {
    fn from(val: T) -> Self {
        Ready(Some(val))
    }
}

impl<T> Task for Ready<T> {
    type Output = T;

    #[inline(always)]
    fn poll_task(&mut self) -> Poll<Self::Output> {
        let val = self.0.take().expect("The task cannot resolve twice");
        Poll::Ready(val)
    }
}

/// Create a task which will immediately return a value of `T`.
pub fn ready<T>(val: T) -> Ready<T> {
    Ready::from(val)
}
