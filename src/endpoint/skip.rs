#![allow(missing_docs)]

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use errors::HttpError;

pub fn skip<E1, E2, A, B: HttpError, C>(e1: E1, e2: E2) -> Skip<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint<A, B>,
    E2: IntoEndpoint<C, B>,
{
    Skip {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Debug, Copy, Clone)]
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
    type Result = E1::Result;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let f1 = try_opt!(self.e1.apply(ctx));
        let _f2 = try_opt!(self.e2.apply(ctx));
        Some(f1)
    }
}
