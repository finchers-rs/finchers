#![allow(missing_docs)]

//! Compatible layor for emulating the standard `Future`.

pub use self::compat::{compat, Compat};
use crate::either::Either;

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
}

impl<T, E> Poll<Result<T, E>> {
    pub fn map_ok<F, U>(self, f: F) -> Poll<Result<U, E>>
    where
        F: FnOnce(T) -> U,
    {
        self.map(|t| t.map(f))
    }

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

macro_rules! poll {
    ($e:expr) => {{
        use $crate::future::Poll;
        match Poll::from($e) {
            Poll::Ready(v) => v,
            Poll::Pending => return Poll::Pending,
        }
    }};
}

pub trait Future {
    type Output;
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

pub fn ready<T>(val: T) -> Ready<T> {
    Ready::from(val)
}

mod compat {
    use futures::{Async, Future, IntoFuture};

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

    pub fn compat<F: IntoFuture>(future: F) -> Compat<F::Future> {
        Compat(IntoFuture::into_future(future))
    }
}
