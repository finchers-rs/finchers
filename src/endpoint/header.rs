#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;
use futures::future::{err, ok, FutureResult};
use endpoint::{Endpoint, EndpointContext, EndpointResult};
use http::{header, EmptyHeader, HttpError, Request};

pub fn header<H>() -> Header<H>
where
    H: header::Header + Clone,
{
    Header {
        _marker: PhantomData,
    }
}

pub struct Header<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for Header<H> {}

impl<H> Clone for Header<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for Header<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Header").finish()
    }
}

impl<H> Endpoint for Header<H>
where
    H: header::Header + Clone,
{
    type Item = H;
    type Error = EmptyHeader;
    type Result = HeaderResult<H>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(HeaderResult {
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct HeaderResult<H>
where
    H: header::Header + Clone,
{
    _marker: PhantomData<fn() -> H>,
}

impl<H> EndpointResult for HeaderResult<H>
where
    H: header::Header + Clone,
{
    type Item = H;
    type Error = EmptyHeader;
    type Future = FutureResult<H, Result<EmptyHeader, HttpError>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        match request.header().cloned() {
            Some(h) => ok(h),
            None => err(Ok(EmptyHeader(H::header_name()).into())),
        }
    }
}

pub fn header_opt<H, E>() -> HeaderOpt<H, E>
where
    H: header::Header + Clone,
{
    HeaderOpt {
        _marker: PhantomData,
    }
}

pub struct HeaderOpt<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Copy for HeaderOpt<H, E> {}

impl<H, E> Clone for HeaderOpt<H, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> fmt::Debug for HeaderOpt<H, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HeaderOpt").finish()
    }
}

impl<H, E> Endpoint for HeaderOpt<H, E>
where
    H: header::Header + Clone,
{
    type Item = Option<H>;
    type Error = E;
    type Result = HeaderOptResult<H, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(HeaderOptResult {
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct HeaderOptResult<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> EndpointResult for HeaderOptResult<H, E>
where
    H: header::Header + Clone,
{
    type Item = Option<H>;
    type Error = E;
    type Future = FutureResult<Option<H>, Result<E, HttpError>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        ok(request.header().cloned())
    }
}
