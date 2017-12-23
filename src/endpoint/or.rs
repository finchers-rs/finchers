#![allow(missing_docs)]

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use task::{Poll, Task, TaskContext};


pub fn or<E1, E2, A, B>(e1: E1, e2: E2) -> Or<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint<A, B>,
    E2: IntoEndpoint<A, B>,
{
    Or {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}


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
    type Task = OrTask<E1::Task, E2::Task>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let mut ctx1 = ctx.clone();
        match self.e1.apply(&mut ctx1) {
            Some(fut) => {
                *ctx = ctx1;
                return Some(OrTask {
                    inner: Either::Left(fut),
                });
            }
            None => {}
        }

        match self.e2.apply(ctx) {
            Some(fut) => Some(OrTask {
                inner: Either::Right(fut),
            }),
            None => None,
        }
    }
}


#[derive(Debug)]
pub struct OrTask<T1, T2>
where
    T1: Task,
    T2: Task<Item = T1::Item, Error = T1::Error>,
{
    inner: Either<T1, T2>,
}

impl<T1, T2> Task for OrTask<T1, T2>
where
    T1: Task,
    T2: Task<Item = T1::Item, Error = T1::Error>,
{
    type Item = T1::Item;
    type Error = T1::Error;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
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
