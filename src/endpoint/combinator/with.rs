#![allow(missing_docs)]

use context::Context;
use endpoint::{Endpoint, EndpointError};

pub fn with<E1, E2>(e1: E1, e2: E2) -> With<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    With { e1, e2 }
}

#[derive(Debug)]
pub struct With<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for With<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = E2::Item;
    type Error = E2::Error;
    type Task = E2::Task;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let _f1 = self.e1.apply(ctx)?;
        let f2 = self.e2.apply(ctx)?;
        Ok(f2)
    }
}
