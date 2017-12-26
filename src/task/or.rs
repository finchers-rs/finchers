use futures::{Future, Poll};
use super::{Task, TaskContext};

#[derive(Debug)]
pub(crate) enum Either<T1, T2> {
    Left(T1),
    Right(T2),
}
pub(crate) use self::Either::*;

#[derive(Debug)]
pub struct Or<T1, T2> {
    pub(crate) inner: Either<T1, T2>,
}

impl<T1, T2> Task for Or<T1, T2>
where
    T1: Task,
    T2: Task<Item = T1::Item, Error = T1::Error>,
{
    type Item = T1::Item;
    type Error = T1::Error;
    type Future = OrFuture<T1::Future, T2::Future>;
    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        match self.inner {
            Left(t) => OrFuture {
                inner: Left(t.launch(ctx)),
            },
            Right(t) => OrFuture {
                inner: Right(t.launch(ctx)),
            },
        }
    }
}

#[derive(Debug)]
pub struct OrFuture<T1, T2> {
    inner: Either<T1, T2>,
}

impl<T1, T2> Future for OrFuture<T1, T2>
where
    T1: Future,
    T2: Future<Item = T1::Item, Error = T1::Error>,
{
    type Item = T1::Item;
    type Error = T1::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner {
            Left(ref mut e) => e.poll(),
            Right(ref mut e) => e.poll(),
        }
    }
}
