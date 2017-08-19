pub mod core;
pub mod header;
pub mod join;
pub mod method;
pub mod param;
pub mod path;

use std::borrow::Cow;
use futures::future::{ok, FutureResult};
use hyper::StatusCode;

use context::Context;
use endpoint::Endpoint;
use errors::{EndpointResult, EndpointErrorKind};


impl<'a> Endpoint for &'a str {
    type Item = ();
    type Future = FutureResult<(), StatusCode>;

    fn apply<'r>(self, mut ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        match ctx.routes.get(0) {
            Some(s) if s == &self => {}
            _ => return Err(EndpointErrorKind::NoRoute.into()),
        }
        ctx.routes.pop_front();
        Ok((ctx, ok(())))
    }
}

impl Endpoint for String {
    type Item = ();
    type Future = FutureResult<(), StatusCode>;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        (&self as &str).apply(ctx)
    }
}

impl<'a> Endpoint for Cow<'a, str> {
    type Item = ();
    type Future = FutureResult<(), StatusCode>;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        (&self as &str).apply(ctx)
    }
}
