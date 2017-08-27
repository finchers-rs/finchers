use futures::{Future, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointResult};
use util::either::Either2;


/// Equivalent to `e1.or(e2)`
pub fn or<E1, E2>(e1: E1, e2: E2) -> Or<E1, E2> {
    Or { e1, e2 }
}


/// The return type of `or(e1, e2)`
#[derive(Debug)]
pub struct Or<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2, T, E> Endpoint for Or<E1, E2>
where
    E1: Endpoint<Item = T, Error = E>,
    E2: Endpoint<Item = T, Error = E>,
{
    type Item = T;
    type Error = E;
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let Or { e1, e2 } = self;

        let mut ctx1 = ctx.clone();
        if let Ok(f) = e1.apply(&mut ctx1) {
            *ctx = ctx1;
            return Ok(OrFuture {
                inner: Either2::E1(f),
            });
        }

        e2.apply(ctx).map(|f| {
            OrFuture {
                inner: Either2::E2(f),
            }
        })
    }
}


#[doc(hidden)]
#[derive(Debug)]
pub struct OrFuture<F1, F2> {
    inner: Either2<F1, F2>,
}

impl<F1, F2, T, E> Future for OrFuture<F1, F2>
where
    F1: Future<Item = T, Error = E>,
    F2: Future<Item = T, Error = E>,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner {
            Either2::E1(ref mut e) => e.poll(),
            Either2::E2(ref mut e) => e.poll(),
        }
    }
}
