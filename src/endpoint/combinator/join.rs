#![allow(missing_docs)]

use std::fmt;
use futures::{Async, Future, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};
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
    type Future = JoinFuture<E1, E2>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let f1 = self.e1.apply(ctx)?;
        let f2 = self.e2.apply(ctx)?;
        Ok(JoinFuture {
            f1: MaybeDone::NotYet(f1),
            f2: MaybeDone::NotYet(f2),
        })
    }
}

pub struct JoinFuture<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    f1: MaybeDone<E1::Future>,
    f2: MaybeDone<E2::Future>,
}

impl<E1, E2> fmt::Debug for JoinFuture<E1, E2>
where
    E1: Endpoint,
    E1::Item: fmt::Debug,
    E1::Future: Future + fmt::Debug,
    E2: Endpoint<Error = E1::Error>,
    E2::Item: fmt::Debug,
    E2::Future: Future + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(stringify!(JoinFuture<E1, E2>))
            .field("f1", &self.f1)
            .field("f2", &self.f2)
            .finish()
    }
}

impl<E1, E2> JoinFuture<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    fn erase(&mut self) {
        self.f1 = MaybeDone::Gone;
        self.f2 = MaybeDone::Gone;
    }
}

impl<E1, E2> Future for JoinFuture<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = (E1::Item, E2::Item);
    type Error = E1::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut all_done = true;

        all_done = all_done && match self.f1.poll() {
            Ok(done) => done,
            Err(e) => {
                self.erase();
                return Err(e);
            }
        };

        all_done = all_done && match self.f2.poll() {
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
