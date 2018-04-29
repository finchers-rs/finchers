//! Components for constructing asynchronous computations which will be returned from "Endpoint"s.
//!
//! # `Outcome`
//!
//! The main trait in this module is "Outcome".
//! This trait is an abstraction of asynchronous computations which will be returned from endpoints.
//! The role of this trait is very close to "futures" and hence its design intentionally resembles
//! "Future". However, some differences are exist for specializing to the purpose of HTTP handling.
//!
//! This trait does not provide any combinators for composing complicate computations.
//! Such combinations are usually performed indirectly by the endpoints or by wrapping the value of
//! "Future".

use either::Either;
use error::Error;
use futures::{Async, Future, IntoFuture};
use input::{Input, RequestBody};
use std::marker::PhantomData;
#[cfg(feature = "nightly")]
use std::ops::Try;

/// All variants which will be returned from "Outcome::poll_outcome".
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

#[cfg(feature = "nightly")]
impl<T> Try for PollOutcome<T> {
    type Ok = T;
    type Error = Option<Error>;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            PollOutcome::Ready(ready) => Ok(ready),
            PollOutcome::Abort(error) => Err(Some(error)),
            PollOutcome::Pending => Err(None),
        }
    }

    fn from_ok(v: Self::Ok) -> Self {
        PollOutcome::Ready(v)
    }

    fn from_error(v: Self::Error) -> Self {
        v.map_or_else(|| PollOutcome::Pending, |e| PollOutcome::Abort(e))
    }
}

/// A helper macro to extract the value from "PollOutcome".
///
/// Typically, this macro is used in the implementation of "Outcome::poll_outcome".
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

/// The contextual information during polling an outcome.
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
///
pub trait Outcome {
    /// The *inner* type of an output which will be returned from this outcome.
    type Output;

    /// Perform polling this outcome and get its result.
    fn poll_outcome(&mut self, cx: &mut Context) -> PollOutcome<Self::Output>;
}

impl<L, R> Outcome for Either<L, R>
where
    L: Outcome,
    R: Outcome<Output = L::Output>,
{
    type Output = L::Output;

    #[inline(always)]
    fn poll_outcome(&mut self, cx: &mut Context) -> PollOutcome<Self::Output> {
        match *self {
            Either::Left(ref mut t) => t.poll_outcome(cx),
            Either::Right(ref mut t) => t.poll_outcome(cx),
        }
    }
}

/// Trait representing the conversion to an "Outcome".
pub trait IntoOutcome {
    /// The type of *output* value.
    type Output;

    /// The type of value to be converted.
    type Outcome: Outcome<Output = Self::Output>;

    /// Perform conversion itself to "Outcome".
    fn into_outcome(self) -> Self::Outcome;
}

// FIXME: replace the trait bound with `core::ops::Async`
impl<F: IntoFuture> IntoOutcome for F {
    type Output = Result<F::Item, F::Error>;
    type Outcome = OutcomeFuture<F::Future>;

    #[inline(always)]
    fn into_outcome(self) -> Self::Outcome {
        future(self)
    }
}

/// A wrapper struct which contains a "Future".
#[derive(Debug)]
pub struct OutcomeFuture<F>(F);

impl<F: Future> From<F> for OutcomeFuture<F> {
    fn from(fut: F) -> Self {
        OutcomeFuture(fut)
    }
}

impl<F: Future> Outcome for OutcomeFuture<F> {
    type Output = Result<F::Item, F::Error>;

    #[inline(always)]
    fn poll_outcome(&mut self, _: &mut Context) -> PollOutcome<Self::Output> {
        match Future::poll(&mut self.0) {
            Ok(Async::Ready(ready)) => PollOutcome::Ready(Ok(ready)),
            Ok(Async::NotReady) => PollOutcome::Pending,
            Err(err) => PollOutcome::Ready(Err(err)),
        }
    }
}

/// Create an outcome from a "Future".
pub fn future<F: IntoFuture>(future: F) -> OutcomeFuture<F::Future> {
    OutcomeFuture::from(IntoFuture::into_future(future))
}

#[derive(Debug)]
pub struct Ready<T>(Option<T>);

impl<T> From<T> for Ready<T> {
    fn from(val: T) -> Self {
        Ready(Some(val))
    }
}

impl<T> Outcome for Ready<T> {
    type Output = T;

    #[inline(always)]
    fn poll_outcome(&mut self, _: &mut Context) -> PollOutcome<Self::Output> {
        let val = self.0.take().expect("The outcome cannot resolve twice");
        PollOutcome::Ready(val)
    }
}

/// Create an outcome which will be immediately resolved as a value of "T".
pub fn ready<T>(val: T) -> Ready<T> {
    Ready::from(val)
}

#[derive(Debug)]
pub struct Abort<T, E> {
    cause: Option<E>,
    _marker: PhantomData<T>,
}

impl<T, E> From<E> for Abort<T, E>
where
    E: Into<Error>,
{
    fn from(cause: E) -> Self {
        Abort {
            cause: Some(cause),
            _marker: PhantomData,
        }
    }
}

impl<T, E> Outcome for Abort<T, E>
where
    E: Into<Error>,
{
    type Output = T;

    #[inline(always)]
    fn poll_outcome(&mut self, _: &mut Context) -> PollOutcome<Self::Output> {
        let cause = self.cause.take().expect("The outcome cannot reject twice");
        PollOutcome::Abort(Into::into(cause))
    }
}

/// Create an outcome which will be immediately rejected as an error value of "E".
pub fn abort<T, E>(cause: E) -> Abort<T, E>
where
    E: Into<Error>,
{
    Abort::from(cause)
}
