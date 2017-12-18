use hyper::Method;

use endpoint::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};


#[derive(Debug, Clone)]
pub struct MatchMethod<E: Endpoint>(Method, E);

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        let f = self.1.apply(ctx)?;
        if ctx.count_remaining_segments() > 0 {
            return Err(EndpointError::Skipped);
        }
        if *ctx.request().method() != self.0 {
            return Err(EndpointError::Skipped);
        }
        Ok(f)
    }
}

macro_rules! generate {
    ($(
        $(#[$doc:meta])*
        ($method:ident, $name:ident),
    )*) => {$(
        $(#[$doc])*
        pub fn $name<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
        where
            E: IntoEndpoint<A, B>,
        {
            MatchMethod(Method::$method, endpoint.into_endpoint())
        }
    )*};
}
generate! {
    (Get, get),
    (Post, post),
    (Put, put),
    (Delete, delete),
    (Head, head),
    (Patch, patch),
}
