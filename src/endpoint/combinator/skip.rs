use context::Context;
use endpoint::{Endpoint, EndpointResult};


/// Equivalent to `e1.skip(e2)`
pub fn skip<E1, E2>(e1: E1, e2: E2) -> Skip<E1, E2> {
    Skip { e1, e2 }
}

/// Return type of `skip(e1, e2)`
#[derive(Debug)]
pub struct Skip<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for Skip<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = E1::Item;
    type Error = E1::Error;
    type Future = E1::Future;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let Skip { e1, e2 } = self;
        let a = e1.apply(ctx)?;
        let _ = e2.apply(ctx)?;
        Ok(a)
    }
}
