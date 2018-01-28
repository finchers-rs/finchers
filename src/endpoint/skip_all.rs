#![allow(missing_docs)]

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use errors::HttpError;

pub fn skip_all<I, E, A, B: HttpError>(iter: I) -> SkipAll<E::Endpoint>
where
    I: IntoIterator<Item = E>,
    E: IntoEndpoint<A, B>,
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
    type Error = E::Error;
    type Result = Result<(), E::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        for endpoint in &self.endpoints {
            let _ = try_opt!(endpoint.apply(ctx));
        }
        Some(Ok(()))
    }
}
