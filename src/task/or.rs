use context::Context;
use super::{Poll, Task};


pub fn left<T1, T2>(t1: T1) -> Or<T1, T2>
where
    T1: Task,
    T2: Task<Item = T1::Item, Error = T1::Error>,
{
    Or {
        inner: Either::Left(t1),
    }
}

pub fn right<T1, T2>(t2: T2) -> Or<T1, T2>
where
    T1: Task,
    T2: Task<Item = T1::Item, Error = T1::Error>,
{
    Or {
        inner: Either::Right(t2),
    }
}


#[derive(Debug)]
pub struct Or<T1, T2>
where
    T1: Task,
    T2: Task<Item = T1::Item, Error = T1::Error>,
{
    inner: Either<T1, T2>,
}

impl<T1, T2> Task for Or<T1, T2>
where
    T1: Task,
    T2: Task<Item = T1::Item, Error = T1::Error>,
{
    type Item = T1::Item;
    type Error = T1::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        match self.inner {
            Either::Left(ref mut e) => e.poll(ctx),
            Either::Right(ref mut e) => e.poll(ctx),
        }
    }
}

#[derive(Debug)]
enum Either<A, B> {
    Left(A),
    Right(B),
}
