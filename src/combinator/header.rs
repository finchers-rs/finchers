//! Definition of endpoints to parse request headers

use std::marker::PhantomData;
use futures::future::{ok, FutureResult};
use hyper::header::{self, Authorization, ContentType};

use context::Context;
use endpoint::Endpoint;
use errors::*;
use request::Body;


#[allow(missing_docs)]
pub struct Header<H>(PhantomData<fn(H) -> H>);

impl<H: header::Header + Clone> Endpoint for Header<H> {
    type Item = H;
    type Future = FutureResult<H, FinchersError>;

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        match ctx.request.header::<H>() {
            Some(h) => Ok((ctx, body, ok(h.clone()))),
            None => Err((FinchersErrorKind::Routing.into(), body)),
        }
    }
}


/// Create a combinator to access a request header
pub fn header<H: header::Header + Clone>() -> Header<H> {
    Header(PhantomData)
}

/// Create a combinator to get the `Authorization` header
pub fn authorization<S: header::Scheme + 'static>() -> Header<Authorization<S>> {
    header()
}

/// Create a combinator to get the `ContentType` header
pub fn content_type() -> Header<ContentType> {
    header()
}
