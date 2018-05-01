use futures::{self, Async};
#[cfg(feature = "nightly")]
use std::ops::Try;

/// A type alias of `Poll<Result<T, E>>`.
pub type PollResult<T, E> = Poll<Result<T, E>>;

/// An enum which indicates whether a value is ready or not.
// FIXME: replace with core::task::Poll
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Poll<T> {
    Ready(T),
    Pending,
}

impl<T> Poll<T> {
    pub fn map<F, U>(self, f: F) -> Poll<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Poll::Pending => Poll::Pending,
            Poll::Ready(t) => Poll::Ready(f(t)),
        }
    }

    pub fn is_pending(&self) -> bool {
        match *self {
            Poll::Pending => true,
            _ => false,
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.is_pending()
    }
}

impl<T, E> Poll<Result<T, E>> {
    pub fn is_ok(&self) -> bool {
        match *self {
            Poll::Ready(Ok(..)) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match *self {
            Poll::Ready(Err(..)) => true,
            _ => false,
        }
    }

    pub fn map_ok<F, U>(self, f: F) -> Poll<Result<U, E>>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Poll::Pending => Poll::Pending,
            Poll::Ready(t) => Poll::Ready(t.map(f)),
        }
    }

    pub fn map_err<F, U>(self, f: F) -> Poll<Result<T, U>>
    where
        F: FnOnce(E) -> U,
    {
        match self {
            Poll::Pending => Poll::Pending,
            Poll::Ready(t) => Poll::Ready(t.map_err(f)),
        }
    }
}

impl<T> From<T> for Poll<T> {
    fn from(ready: T) -> Poll<T> {
        Poll::Ready(ready)
    }
}

impl<T> From<Async<T>> for Poll<T> {
    fn from(v: Async<T>) -> Self {
        match v {
            Async::NotReady => Poll::Pending,
            Async::Ready(v) => Poll::Ready(v),
        }
    }
}

impl<T> Into<Async<T>> for Poll<T> {
    fn into(self) -> Async<T> {
        match self {
            Poll::Pending => Async::NotReady,
            Poll::Ready(v) => Async::Ready(v),
        }
    }
}

impl<T, E> From<futures::Poll<T, E>> for Poll<Result<T, E>> {
    fn from(v: futures::Poll<T, E>) -> Self {
        match v {
            Ok(Async::NotReady) => Poll::Pending,
            Ok(Async::Ready(ok)) => Poll::Ready(Ok(ok)),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T, E> Into<futures::Poll<T, E>> for Poll<Result<T, E>> {
    fn into(self) -> futures::Poll<T, E> {
        match self {
            Poll::Pending => Ok(Async::NotReady),
            Poll::Ready(Ok(ok)) => Ok(Async::Ready(ok)),
            Poll::Ready(Err(err)) => Err(err),
        }
    }
}

#[cfg(feature = "nightly")]
impl<T, E> Try for Poll<Result<T, E>> {
    type Ok = T;
    type Error = PollError<E>;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Poll::Pending => Err(PollError::Pending),
            Poll::Ready(Ok(ok)) => Ok(ok),
            Poll::Ready(Err(e)) => Err(PollError::Error(e)),
        }
    }

    fn from_ok(v: Self::Ok) -> Self {
        Poll::Ready(Ok(v))
    }

    fn from_error(v: Self::Error) -> Self {
        match v {
            PollError::Pending => Poll::Pending,
            PollError::Error(err) => Poll::Ready(Err(err)),
        }
    }
}

// An opaque type for implementation of Try
#[cfg(feature = "nightly")]
pub enum PollError<E> {
    Pending,
    Error(E),
}

/// A helper macro for extracting the value of `Poll<T>`.
#[macro_export]
macro_rules! poll {
    ($e:expr) => {{
        use $crate::Poll;
        match Poll::from($e) {
            Poll::Ready(v) => v,
            Poll::Pending => return Poll::Pending,
        }
    }};
}

/// A helper macro for extracting the successful value of `PollResult<T, E>`.
#[macro_export]
macro_rules! poll_result {
    ($e:expr) => {{
        use $crate::Poll;
        match Poll::from($e) {
            Poll::Ready(Ok(v)) => v,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(Into::into(e))),
            Poll::Pending => return Poll::Pending,
        }
    }};
}
