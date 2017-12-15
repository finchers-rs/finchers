#![allow(missing_docs)]

use std::fmt;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task::{Async, Poll, Task};
use super::maybe_done::MaybeDone;

// TODO: add Join3, Join4, Join5

pub fn join<E1, E2>(e1: E1, e2: E2) -> Join<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    Join { e1, e2 }
}

#[derive(Debug)]
pub struct Join<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for Join<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = (E1::Item, E2::Item);
    type Error = E1::Error;
    type Task = JoinTask<E1, E2>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let f1 = self.e1.apply(ctx)?;
        let f2 = self.e2.apply(ctx)?;
        Ok(JoinTask {
            f1: MaybeDone::NotYet(f1),
            f2: MaybeDone::NotYet(f2),
        })
    }
}

pub struct JoinTask<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    f1: MaybeDone<E1::Task>,
    f2: MaybeDone<E2::Task>,
}

impl<E1, E2> fmt::Debug for JoinTask<E1, E2>
where
    E1: Endpoint,
    E1::Item: fmt::Debug,
    E1::Task: Task + fmt::Debug,
    E2: Endpoint<Error = E1::Error>,
    E2::Item: fmt::Debug,
    E2::Task: Task + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(stringify!(JoinFuture<E1, E2>))
            .field("f1", &self.f1)
            .field("f2", &self.f2)
            .finish()
    }
}

impl<E1, E2> JoinTask<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    fn erase(&mut self) {
        self.f1 = MaybeDone::Gone;
        self.f2 = MaybeDone::Gone;
    }
}

impl<E1, E2> Task for JoinTask<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = (E1::Item, E2::Item);
    type Error = E1::Error;

    fn poll(&mut self, ctx: &mut Context) -> Poll<Self::Item, Self::Error> {
        let mut all_done = true;

        all_done = all_done && match self.f1.poll(ctx) {
            Ok(done) => done,
            Err(e) => {
                self.erase();
                return Err(e);
            }
        };

        all_done = all_done && match self.f2.poll(ctx) {
            Ok(done) => done,
            Err(e) => {
                self.erase();
                return Err(e);
            }
        };

        if all_done {
            Ok(Async::Ready((self.f1.take(), self.f2.take())))
        } else {
            Ok(Async::NotReady)
        }
    }
}
