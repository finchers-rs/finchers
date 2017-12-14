#![allow(missing_docs)]

use context::Context;
use endpoint::{Endpoint, EndpointError};


pub fn skip<E1, E2>(e1: E1, e2: E2) -> Skip<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    Skip { e1, e2 }
}

#[derive(Debug)]
pub struct Skip<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for Skip<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = E1::Item;
    type Error = E1::Error;
    type Future = E1::Future;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        let f1 = self.e1.apply(ctx)?;
        let _f2 = self.e2.apply(ctx)?;
        Ok(f1)
    }
}
