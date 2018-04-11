use finchers_core::HttpError;
use finchers_core::endpoint::{Context, Endpoint, task::{self, Async, Future, Poll}};
use futures;
use futures::IntoFuture;
use std::mem;

pub fn new<E, F, R>(endpoint: E, f: F) -> AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R + Clone + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    AndThen { endpoint, f }
}

#[derive(Copy, Clone, Debug)]
pub struct AndThen<E, F> {
    endpoint: E,
    f: F,
}

impl<E, F, R> Endpoint for AndThen<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R + Clone + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    type Item = R::Item;
    type Future = AndThenFuture<E::Future, F, R>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let future = self.endpoint.apply(cx)?;
        Some(AndThenFuture::First(future, self.f.clone()))
    }
}

#[derive(Debug)]
pub enum AndThenFuture<T, F, R>
where
    T: Future,
    F: FnOnce(T::Item) -> R + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    First(T, F),
    Second(R::Future),
    Done,
}

impl<T, F, R> Future for AndThenFuture<T, F, R>
where
    T: Future,
    F: FnOnce(T::Item) -> R + Send,
    R: IntoFuture,
    R::Future: Send,
    R::Error: HttpError,
{
    type Item = R::Item;

    fn poll(&mut self, cx: &mut task::Context) -> Poll<Self::Item> {
        use self::AndThenFuture::*;
        loop {
            // TODO: optimize
            match mem::replace(self, Done) {
                First(mut fut, f) => match fut.poll(cx)? {
                    Async::NotReady => {
                        *self = First(fut, f);
                        return Ok(Async::NotReady);
                    }
                    Async::Ready(r) => {
                        *self = Second(f(r).into_future());
                        continue;
                    }
                },
                Second(mut fut) => match futures::Future::poll(&mut fut)? {
                    Async::NotReady => {
                        *self = Second(fut);
                        return Ok(Async::NotReady);
                    }
                    Async::Ready(item) => return Ok(Async::Ready(item)),
                },
                Done => panic!(),
            }
        }
    }
}
