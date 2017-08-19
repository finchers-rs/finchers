use hyper::Method;

use context::Context;
use endpoint::Endpoint;
use errors::{EndpointResult, EndpointErrorKind};
use request::Body;


pub struct MatchMethod<E>(Method, E);

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        if *ctx.request.method() != self.0 {
            return Err((EndpointErrorKind::InvalidMethod.into(), body));
        }
        self.1.apply(ctx, body)
    }
}

pub fn get<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Get, endpoint)
}

pub fn post<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Post, endpoint)
}
