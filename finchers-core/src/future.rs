#![allow(missing_docs)]

//! Compatible layor for emulating the standard `Future`.

pub use self::compat::{compat, Compat};

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

pub trait Future {
    type Output;
    fn poll(&mut self) -> Poll<Self::Output>;
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

pub trait TryFuture {
    type Ok;
    type Error;

    fn try_poll(&mut self) -> Poll<Result<Self::Ok, Self::Error>>;
}

impl<F, T, E> TryFuture for F
where
    F: Future<Output = Result<T, E>>,
{
    type Ok = T;
    type Error = E;

    #[inline]
    fn try_poll(&mut self) -> Poll<Result<Self::Ok, Self::Error>> {
        self.poll()
    }
}

macro_rules! poll {
    ($e:expr) => {{
        use $crate::future::Poll;
        match $e {
            Poll::Ready(x) => x,
            Poll::Pending => return Poll::Pending,
        }
    }};
}

macro_rules! try_poll {
    ($e:expr) => {{
        use $crate::future::Poll;
        match $e {
            Poll::Ready(Ok(x)) => x,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(Into::into(e))),
            Poll::Pending => return Poll::Pending,
        }
    }};
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
