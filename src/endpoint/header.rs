//! Definition of endpoints to parse request headers

use std::marker::PhantomData;
use futures::future::{ok, FutureResult};
use hyper::header::{self, Authorization, ContentType};

use context::Context;
use endpoint::{Endpoint, EndpointError};

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Header<H, E>(PhantomData<fn() -> (H, E)>);

impl<H, E> Clone for Header<H, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> Copy for Header<H, E> {}

impl<H: header::Header + Clone, E> Endpoint for Header<H, E> {
    type Item = H;
    type Error = E;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        ctx.request()
            .header()
            .cloned()
            .map(ok)
            .ok_or(EndpointError::EmptyHeader)
    }
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct HeaderOpt<H, E>(PhantomData<fn() -> (H, E)>);

impl<H, E> Clone for HeaderOpt<H, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> Copy for HeaderOpt<H, E> {}


impl<H: header::Header + Clone, E> Endpoint for HeaderOpt<H, E> {
    type Item = Option<H>;
    type Error = E;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        Ok(ok(ctx.request().header().cloned()))
    }
}


/// Create an endpoint matches the value of a request header
pub fn header<H: header::Header + Clone, E>() -> Header<H, E> {
    Header(PhantomData)
}

/// Create an endpoint matches the value of a request header, which the value may not exist
pub fn header_opt<H: header::Header + Clone, E>() -> HeaderOpt<H, E> {
    HeaderOpt(PhantomData)
}


/// Equivalent to `header::<Authorization<S>>()`
pub fn authorization<S: header::Scheme + 'static, E>() -> Header<Authorization<S>, E> {
    header()
}

/// Equivalent to `header::<ContentType>()`
pub fn content_type<E>() -> Header<ContentType, E> {
    header()
}
