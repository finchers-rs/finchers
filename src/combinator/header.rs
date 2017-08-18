use std::marker::PhantomData;
use futures::future::{ok, FutureResult};
use hyper::header::{self, Authorization, ContentType};

use context::Context;
use endpoint::Endpoint;
use errors::{EndpointResult, EndpointErrorKind};


pub struct Header<H>(PhantomData<fn(H) -> H>);

pub fn header<H: header::Header + Clone>() -> Header<H> {
    Header(PhantomData)
}

impl<H: header::Header + Clone> Endpoint for Header<H> {
    type Item = H;
    type Error = ();
    type Future = FutureResult<H, ()>;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        match ctx.request.header::<H>() {
            Some(h) => Ok((ctx, ok(h.clone()))),
            None => Err(EndpointErrorKind::NoRoute.into()),
        }
    }
}

pub fn authorization<S: header::Scheme + 'static>() -> Header<Authorization<S>> {
    header()
}

pub fn content_type() -> Header<ContentType> {
    header()
}
