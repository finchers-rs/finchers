use context::Context;
use endpoint::{Endpoint, EndpointResult};
use super::either::Either2;


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

impl<E1, E2, E> Endpoint for Or<E1, E2>
where
    E1: Endpoint<Error = E>,
    E2: Endpoint<Error = E>,
{
    type Item = Either2<E1::Item, E2::Item>;
    type Error = E;
    type Future = Either2<E1::Future, E2::Future>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let Or { e1, e2 } = self;

        let mut ctx1 = ctx.clone();
        if let Ok(f) = e1.apply(&mut ctx1) {
            *ctx = ctx1;
            return Ok(Either2::E1(f));
        }

        e2.apply(ctx).map(Either2::E2)
    }
}
