use futures::{Future, Poll};

use context::Context;
use endpoint::{Endpoint, EndpointError};


/// Equivalent to `e1.or(e2)`
pub fn or<E1, E2>(e1: E1, e2: E2) -> Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Item = E1::Item, Error = E1::Error>,
{
    Or { e1, e2 }
}


/// The return type of `or(e1, e2)`
#[derive(Debug)]
pub struct Or<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Item = E1::Item, Error = E1::Error>,
{
    type Item = E1::Item;
    type Error = E1::Error;
    type Future = OrFuture<E1, E2>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let mut ctx1 = ctx.clone();
        match self.e1.apply(&mut ctx1) {
            Ok(fut) => {
                *ctx = ctx1;
                return Ok(OrFuture {
                    inner: Either::A(fut),
                });
            }
            Err(..) => {}
        }

        match self.e2.apply(ctx) {
            Ok(fut) => Ok(OrFuture {
                inner: Either::B(fut),
            }),
            Err(err) => Err(err),
        }
    }
}


#[derive(Debug)]
pub struct OrFuture<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Item = E1::Item, Error = E1::Error>,
{
    inner: Either<E1::Future, E2::Future>,
}

impl<E1, E2> Future for OrFuture<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Item = E1::Item, Error = E1::Error>,
{
    type Item = E1::Item;
    type Error = E1::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner {
            Either::A(ref mut e) => e.poll(),
            Either::B(ref mut e) => e.poll(),
        }
    }
}

#[derive(Debug)]
enum Either<A, B> {
    A(A),
    B(B),
}
