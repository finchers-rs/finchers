use context::Context;
use endpoint::{Endpoint, EndpointError, IntoEndpoint};
use task;


/// Equivalent to `e1.or(e2)`
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
