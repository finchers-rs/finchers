#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase, IntoEndpoint};

pub fn new<E1, E2>(e1: E1, e2: E2) -> Left<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
{
    Left {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Left<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> EndpointBase for Left<E1, E2>
where
    E1: EndpointBase,
    E2: EndpointBase,
{
    type Output = E1::Output;
    type Future = E1::Future;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let f1 = self.e1.apply(cx)?;
        drop(self.e2.apply(cx)?);
        Some(f1)
    }
}
