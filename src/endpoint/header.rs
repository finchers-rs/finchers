//! Definition of endpoints to parse request headers

use std::marker::PhantomData;
use futures::future::{ok, FutureResult};
use hyper::header::{self, Authorization, ContentType};

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};
use errors::*;


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Header<H>(PhantomData<fn(H) -> H>);

impl<H> Clone for Header<H> {
    fn clone(&self) -> Header<H> {
        Header(PhantomData)
    }
}

impl<H> Copy for Header<H> {}

impl<H: header::Header + Clone> Endpoint for Header<H> {
    type Item = H;
    type Future = FutureResult<H, FinchersError>;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        match ctx.request.header::<H>() {
            Some(h) => Ok(ok(h.clone())),
            None => Err(EndpointError::EmptyHeader),
        }
    }
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct HeaderOpt<H>(PhantomData<fn() -> H>);

impl<H> Clone for HeaderOpt<H> {
    fn clone(&self) -> HeaderOpt<H> {
        HeaderOpt(PhantomData)
    }
}

impl<H> Copy for HeaderOpt<H> {}


impl<H: header::Header + Clone> Endpoint for HeaderOpt<H> {
    type Item = Option<H>;
    type Future = FutureResult<Option<H>, FinchersError>;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        match ctx.request.header::<H>() {
            Some(h) => Ok(ok(Some(h.clone()))),
            None => Ok(ok(None)),
        }
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
