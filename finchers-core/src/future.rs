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

/// An enum which indicates whether a value is ready or not.
// FIXME: replace with core::task::Poll
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Poll<T> {
    /// The task has just been finished with a returned value of `T`.
    Ready(T),

    /// The task is not ready and should be scheduled to be awoken by the executor.
    Pending,
}

impl<T> Poll<T> {
    /// Return whether the value is `Pending`.
    pub fn is_pending(&self) -> bool {
        match *self {
            Poll::Pending => true,
            _ => false,
        }
    }

    /// Return whether the value is `Ready`.
    pub fn is_ready(&self) -> bool {
        !self.is_pending()
    }

    /// Maps the value to a new type with given function.
    pub fn map<F, U>(self, f: F) -> Poll<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Poll::Pending => Poll::Pending,
            Poll::Ready(t) => Poll::Ready(f(t)),
        }
    }
}

impl<T, E> Poll<Result<T, E>> {
    /// Return whether the value is `Ready(Ok(t))`.
    pub fn is_ok(&self) -> bool {
        match *self {
            Poll::Ready(Ok(..)) => true,
            _ => false,
        }
    }

    /// Return whether the value is `Ready(Err(t))`.
    pub fn is_err(&self) -> bool {
        match *self {
            Poll::Ready(Err(..)) => true,
            _ => false,
        }
    }

    /// Maps the value to a new type with given function if the value is `Ok`.
    pub fn map_ok<F, U>(self, f: F) -> Poll<Result<U, E>>
    where
        F: FnOnce(T) -> U,
    {
        self.map(|t| t.map(f))
    }

    /// Maps the value to a new type with given function if the value is `Err`.
    pub fn map_err<F, U>(self, f: F) -> Poll<Result<T, U>>
    where
        F: FnOnce(E) -> U,
    {
        self.map(|t| t.map_err(f))
    }
}

impl<T> From<T> for Poll<T> {
    fn from(ready: T) -> Poll<T> {
        Poll::Ready(ready)
    }
}

/// A helper macro for extracting the value of `Poll<T>`.
macro_rules! poll {
    ($e:expr) => {{
        use $crate::future::Poll;
        match Poll::from($e) {
            Poll::Ready(v) => v,
            Poll::Pending => return Poll::Pending,
        }
    }};
}

/// Trait representing the asynchronous computation after applying the endpoints.
///
/// See the module level documentation for details.
pub trait Future {
    /// The *inner* type of an output which will be returned from this task.
    type Output;

    /// Perform polling this task and get its result.
    fn poll(&mut self) -> Poll<Self::Output>;
}

impl<L, R> Future for Either<L, R>
where
    L: Future,
    R: Future,
{
    type Output = Either<L::Output, R::Output>;

    #[inline(always)]
    fn poll(&mut self) -> Poll<Self::Output> {
        match *self {
            Either::Left(ref mut t) => t.poll().map(Either::Left),
            Either::Right(ref mut t) => t.poll().map(Either::Right),
        }
    }
}

/// A `Task` which will immediately return a value of `T`.
#[derive(Debug)]
pub struct Ready<T>(Option<T>);

impl<T> From<T> for Ready<T> {
    fn from(val: T) -> Self {
        Ready(Some(val))
    }
}

impl<T> Future for Ready<T> {
    type Output = T;

    #[inline(always)]
    fn poll(&mut self) -> Poll<Self::Output> {
        Poll::Ready(self.0.take().expect("The task cannot resolve twice"))
    }
}

/// Create a task which will immediately return a value of `T`.
pub fn ready<T>(val: T) -> Ready<T> {
    Ready::from(val)
}

mod compat {
    use futures::{Async, Future, IntoFuture};

    /// A helper struct which wraps a `Future` and provides the implementation of `Task`.
    #[derive(Debug)]
    pub struct Compat<F>(F);

    impl<F: futures::Future> From<F> for Compat<F> {
        fn from(fut: F) -> Self {
            Compat(fut)
        }
    }

    impl<F: Future> super::Future for Compat<F> {
        type Output = Result<F::Item, F::Error>;

        #[inline(always)]
        fn poll(&mut self) -> super::Poll<Self::Output> {
            match Future::poll(&mut self.0) {
                Ok(Async::Ready(ready)) => super::Poll::Ready(Ok(ready)),
                Ok(Async::NotReady) => super::Poll::Pending,
                Err(err) => super::Poll::Ready(Err(err)),
            }
        }
    }

    /// Create a task from a `Future`.
    pub fn compat<F: IntoFuture>(future: F) -> Compat<F::Future> {
        Compat(IntoFuture::into_future(future))
    }
}
pub use self::compat::{compat, Compat};
