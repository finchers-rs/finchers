//! Definition of endpoints to parse request headers

use std::marker::PhantomData;
use futures::future::{ok, FutureResult};
use hyper::header::{self, Authorization, ContentType};

use context::Context;
use endpoint::Endpoint;
use errors::*;


#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Header<H>(PhantomData<fn(H) -> H>);

impl<H: header::Header + Clone> Endpoint for Header<H> {
    type Item = H;
    type Future = FutureResult<H, FinchersError>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let result = match ctx.request.header::<H>() {
            Some(h) => Ok(ok(h.clone())),
            None => Err(FinchersErrorKind::NotFound.into()),
        };
        (ctx, result)
    }
}


#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct HeaderOpt<H>(PhantomData<fn() -> H>);

impl<H: header::Header + Clone> Endpoint for HeaderOpt<H> {
    type Item = Option<H>;
    type Future = FutureResult<Option<H>, FinchersError>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let result = match ctx.request.header::<H>() {
            Some(h) => Ok(ok(Some(h.clone()))),
            None => Ok(ok(None)),
        };
        (ctx, result)
    }
}


/// Create an endpoint matches the value of a request header
pub fn header<H: header::Header + Clone>() -> Header<H> {
    Header(PhantomData)
}

/// Create an endpoint matches the value of a request header, which the value may not exist
pub fn header_opt<H: header::Header + Clone>() -> HeaderOpt<H> {
    HeaderOpt(PhantomData)
}


/// Equivalent to `header::<Authorization<S>>()`
pub fn authorization<S: header::Scheme + 'static>() -> Header<Authorization<S>> {
    header()
}

/// Equivalent to `header::<ContentType>()`
pub fn content_type() -> Header<ContentType> {
    header()
}
