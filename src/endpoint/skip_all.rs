#![allow(missing_docs)]

use core::NeverReturn;
use endpoint::{Endpoint, EndpointContext, Input, IntoEndpoint};

pub fn skip_all<I>(iter: I) -> SkipAll<<I::Item as IntoEndpoint>::Endpoint>
where
    I: IntoIterator,
    I::Item: IntoEndpoint,
{
    SkipAll {
        endpoints: iter.into_iter().map(|e| e.into_endpoint()).collect(),
    }
}

#[derive(Debug, Clone)]
pub struct SkipAll<E: Endpoint> {
    endpoints: Vec<E>,
}

impl<E: Endpoint> Endpoint for SkipAll<E> {
    type Item = ();
    type Result = Result<(), NeverReturn>;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result> {
        for endpoint in &self.endpoints {
            let _ = try_opt!(endpoint.apply(input, ctx));
        }
        Some(Ok(()))
    }
}
