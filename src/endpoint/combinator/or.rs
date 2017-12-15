use context::Context;
use endpoint::{Endpoint, EndpointError};
use task;


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
    type Task = task::Or<E1::Task, E2::Task>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Task, EndpointError> {
        let mut ctx1 = ctx.clone();
        match self.e1.apply(&mut ctx1) {
            Ok(fut) => {
                *ctx = ctx1;
                return Ok(task::left(fut));
            }
            Err(..) => {}
        }

        match self.e2.apply(ctx) {
            Ok(fut) => Ok(task::right(fut)),
            Err(err) => Err(err),
        }
    }
}
