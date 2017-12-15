#![allow(missing_docs)]

use std::fmt;

use context::Context;
use super::{Async, Poll, Task};
use super::maybe_done::MaybeDone;

// TODO: add Join3, Join4, Join5

pub fn join<T1, T2>(t1: T1, t2: T2) -> Join<T1, T2>
where
    T1: Task,
    T2: Task<Error = T1::Error>,
{
    Join {
        t1: MaybeDone::NotYet(t1),
        t2: MaybeDone::NotYet(t2),
    }
}


pub struct Join<T1, T2>
where
    T1: Task,
    T2: Task<Error = T1::Error>,
{
    t1: MaybeDone<T1>,
    t2: MaybeDone<T2>,
}

impl<T1, T2> fmt::Debug for Join<T1, T2>
where
    T1: Task + fmt::Debug,
    T1::Item: fmt::Debug,
    T2: Task<Error = T1::Error> + fmt::Debug,
    T2::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(stringify!(Join<T1, T2>))
            .field("f1", &self.t1)
            .field("f2", &self.t2)
            .finish()
    }
}

impl<T1, T2> Join<T1, T2>
where
    T1: Task,
    T2: Task<Error = T1::Error>,
{
    fn erase(&mut self) {
        self.t1 = MaybeDone::Gone;
        self.t2 = MaybeDone::Gone;
    }
}

impl<T1, T2> Task for Join<T1, T2>
where
    T1: Task,
    T2: Task<Error = T1::Error>,
{
    type Item = (T1::Item, T2::Item);
    type Error = T1::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let mut all_done = true;

        all_done = all_done && match self.t1.poll(ctx) {
            Ok(done) => done,
            Err(e) => {
                self.erase();
                return Err(e);
            }
        };

        all_done = all_done && match self.t2.poll(ctx) {
            Ok(done) => done,
            Err(e) => {
                self.erase();
                return Err(e);
            }
        };

        if all_done {
            Ok(Async::Ready((self.t1.take(), self.t2.take())))
        } else {
            Ok(Async::NotReady)
        }
    }
}
