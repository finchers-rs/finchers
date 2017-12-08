use context::Context;
use endpoint::{Endpoint, EndpointResult};


/// Equivalent to `e1.with(e2)`
pub fn with<E1, E2, E>(e1: E1, e2: E2) -> With<E1, E2>
where
    E1: Endpoint<Error = E>,
    E2: Endpoint<Error = E>,
{
    With { e1, e2 }
}

/// The return type of `with(e1,e2)`
#[derive(Debug)]
pub struct With<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for With<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = E2::Item;
    type Error = E2::Error;
    type Future = E2::Future;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let With { e1, e2 } = self;
        let _ = e1.apply(ctx)?;
        let b = e2.apply(ctx)?;
        Ok(b)
    }
}
