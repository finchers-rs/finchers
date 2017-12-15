#![allow(missing_docs)]

use context::Context;
use endpoint::{Endpoint, EndpointError};
use task;


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
    type Task = task::Join<E1::Task, E2::Task>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let f1 = self.e1.apply(ctx)?;
        let f2 = self.e2.apply(ctx)?;
        Ok(task::join(f1, f2))
    }
}
