#![allow(missing_docs)]

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use request::Input;

pub fn skip<E1, E2>(e1: E1, e2: E2) -> Skip<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
{
    Skip {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Debug, Copy, Clone)]
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
    type Future = E1::Future;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        let f1 = self.e1.apply(input, ctx)?;
        let _f2 = self.e2.apply(input, ctx)?;
        Some(f1)
    }
}
