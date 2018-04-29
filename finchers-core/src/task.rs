//! Components for constructing asynchronous computations which will be returned from "Endpoint"s.
//!
//! The main trait in this module is `Task`.
//! This trait is an abstraction of asynchronous computations which will be returned from endpoints.
//! The role of this trait is very close to "futures" and hence its design intentionally resembles
//! `Future`. However, some differences are exist for specializing to the purpose of HTTP handling.
//!
//! This trait does not provide any combinators for composing complicate computations.
//! Such combinations are usually performed indirectly by the endpoints or by wrapping the value of
//! `Future`.

use either::Either;
use futures::{Async, Future, IntoFuture};
#[cfg(feature = "nightly")]
use std::ops::Try;

use error::Error;
use input::{Input, RequestBody};
use never::Never;

/// The enum indicating the progress of `Task`.
#[derive(Debug)]
pub enum PollTask<T> {
    /// The task has not ready yet.
    Pending,
    /// The task has returned a value.
    Ready(T),
    /// The task aborted with some reason.
    Aborted(Error),
}

impl<T> PollTask<T> {
    pub fn map<F, U>(self, f: F) -> PollTask<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            PollTask::Pending => PollTask::Pending,
            PollTask::Ready(t) => PollTask::Ready(f(t)),
            PollTask::Aborted(e) => PollTask::Aborted(e),
        }
    }

    pub fn is_pending(&self) -> bool {
        match *self {
            PollTask::Pending => true,
            _ => false,
        }
    }

    pub fn is_ready(&self) -> bool {
        match *self {
            PollTask::Ready(..) => true,
            _ => false,
        }
    }

    pub fn is_aborted(&self) -> bool {
        match *self {
            PollTask::Aborted(..) => true,
            _ => false,
        }
    }
}

impl<T> From<Async<T>> for PollTask<T> {
    fn from(val: Async<T>) -> Self {
        match val {
            Async::NotReady => PollTask::Pending,
            Async::Ready(val) => PollTask::Ready(val),
        }
    }
}

impl<T, E> From<Result<Async<T>, E>> for PollTask<T>
where
    E: Into<Error>,
{
    fn from(val: Result<Async<T>, E>) -> Self {
        match val {
            Ok(Async::NotReady) => PollTask::Pending,
            Ok(Async::Ready(val)) => PollTask::Ready(val),
            Err(e) => PollTask::Aborted(Into::into(e)),
        }
    }
}

#[cfg(feature = "nightly")]
impl<T> Try for PollTask<T> {
    type Ok = T;
    type Error = Option<Error>;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            PollTask::Ready(ready) => Ok(ready),
            PollTask::Aborted(error) => Err(Some(error)),
            PollTask::Pending => Err(None),
        }
    }

    fn from_ok(v: Self::Ok) -> Self {
        PollTask::Ready(v)
    }

    fn from_error(v: Self::Error) -> Self {
        v.map_or_else(|| PollTask::Pending, |e| PollTask::Aborted(e))
    }
}

/// A helper macro to extract the value from "PollTask".
///
/// Typically, this macro is used in the implementation of "Task::poll_task".
#[macro_export]
macro_rules! try_ready_task {
    ($e:expr) => {
        match PollTask::from($e) {
            PollTask::Ready(v) => v,
            PollTask::Aborted(e) => return PollTask::Aborted(e),
            PollTask::Pending => return PollTask::Pending,
        }
    };
}

/// The contextual information during polling an task.
pub struct Context<'a> {
    input: &'a Input,
    body: &'a mut Option<RequestBody>,
    // FIXME: add `futures::task::Context`
}

impl<'a> Context<'a> {
    /// Create an instance of "Context" from components.
    #[inline]
    pub fn new(input: &'a Input, body: &'a mut Option<RequestBody>) -> Context<'a> {
        Context { input, body }
    }

    /// Return the reference to "Input" at the current request.
    #[inline]
    pub fn input(&self) -> &Input {
        self.input
    }

    /// Take the instance of "RequestBody" at the current request if available.
    #[inline]
    pub fn body(&mut self) -> Option<RequestBody> {
        self.body.take()
    }
}

/// Trait representing the asynchronous computation after applying the endpoints.
///
/// See the module level documentation for details.
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

    #[inline(always)]
    fn poll_task(&mut self, cx: &mut Context) -> PollTask<Self::Output> {
        match *self {
            Either::Left(ref mut t) => t.poll_task(cx),
            Either::Right(ref mut t) => t.poll_task(cx),
        }
    }
}

/// Trait representing the conversion to a "Task".
pub trait IntoTask {
    /// The type of *output* value.
    type Output;

    /// The type of value to be converted.
    type Task: Task<Output = Self::Output>;

    /// Perform conversion itself to "Task".
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
    fn poll_task(&mut self, _: &mut Context) -> PollTask<Self::Output> {
        match Future::poll(&mut self.0) {
            Ok(Async::Ready(ready)) => PollTask::Ready(Ok(ready)),
            Ok(Async::NotReady) => PollTask::Pending,
            Err(err) => PollTask::Ready(Err(err)),
        }
    }
}

/// Create a task from a `Future`.
pub fn future<F: IntoFuture>(future: F) -> TaskFuture<F::Future> {
    TaskFuture::from(IntoFuture::into_future(future))
}

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
    fn poll_task(&mut self, _: &mut Context) -> PollTask<Self::Output> {
        let val = self.0.take().expect("The task cannot resolve twice");
        PollTask::Ready(val)
    }
}

/// Create a task which will immediately return a value of `T`.
pub fn ready<T>(val: T) -> Ready<T> {
    Ready::from(val)
}

#[derive(Debug)]
pub struct Abort<E> {
    cause: Option<E>,
}

impl<E: Into<Error>> From<E> for Abort<E> {
    fn from(cause: E) -> Self {
        Abort { cause: Some(cause) }
    }
}

impl<E: Into<Error>> Task for Abort<E> {
    type Output = Never;

    #[inline(always)]
    fn poll_task(&mut self, _: &mut Context) -> PollTask<Self::Output> {
        let cause = self.cause.take().expect("The task cannot reject twice");
        PollTask::Aborted(Into::into(cause))
    }
}

/// Create a task which will immediately abort the computation with an error value of `E`.
pub fn abort<E: Into<Error>>(cause: E) -> Abort<E> {
    Abort::from(cause)
}
