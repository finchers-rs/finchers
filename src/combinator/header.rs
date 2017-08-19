use std::marker::PhantomData;
use futures::future::{ok, FutureResult};
use hyper::StatusCode;
use hyper::header::{self, Authorization, ContentType};

use context::Context;
use endpoint::Endpoint;
use errors::{EndpointResult, EndpointErrorKind};
use request::Body;


pub struct Header<H>(PhantomData<fn(H) -> H>);

pub fn header<H: header::Header + Clone>() -> Header<H> {
    Header(PhantomData)
}

impl<H: header::Header + Clone> Endpoint for Header<H> {
    type Item = H;
    type Future = FutureResult<H, StatusCode>;

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        match ctx.request.header::<H>() {
            Some(h) => Ok((ctx, body, ok(h.clone()))),
            None => Err((EndpointErrorKind::NoRoute.into(), body)),
        }
    }
}

pub fn authorization<S: header::Scheme + 'static>() -> Header<Authorization<S>> {
    header()
}

pub fn content_type() -> Header<ContentType> {
    header()
}
