#![allow(missing_docs)]

use std::fmt;
use std::error::Error;
use std::marker::PhantomData;
use futures::future::{err, ok, FutureResult};
use endpoint::{Endpoint, EndpointContext, EndpointResult};
use errors::StdErrorResponseBuilder;
use http::{self, header, IntoResponse, Request, Response};

pub fn header<H: header::Header, E>() -> Header<H, E> {
    Header {
        _marker: PhantomData,
    }
}

pub struct Header<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Copy for Header<H, E> {}

impl<H, E> Clone for Header<H, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> fmt::Debug for Header<H, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Header").finish()
    }
}

impl<H: header::Header, E> Endpoint for Header<H, E> {
    type Item = H;
    type Error = E;
    type Result = HeaderResult<H, E>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        if ctx.headers().has::<H>() {
            Some(HeaderResult {
                _marker: PhantomData,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct HeaderResult<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H: header::Header, E> EndpointResult for HeaderResult<H, E> {
    type Item = H;
    type Error = E;
    type Future = FutureResult<H, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        ok(request.headers_mut().remove().expect(&format!(
            "The value of header {} has already taken",
            H::header_name()
        )))
    }
}

pub fn header_req<H: header::Header>() -> HeaderRequired<H> {
    HeaderRequired {
        _marker: PhantomData,
    }
}

pub struct HeaderRequired<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for HeaderRequired<H> {}

impl<H> Clone for HeaderRequired<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for HeaderRequired<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HeaderRequired").finish()
    }
}

impl<H: header::Header> Endpoint for HeaderRequired<H> {
    type Item = H;
    type Error = EmptyHeader<H>;
    type Result = HeaderRequiredResult<H>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(HeaderRequiredResult {
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct HeaderRequiredResult<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: header::Header> EndpointResult for HeaderRequiredResult<H> {
    type Item = H;
    type Error = EmptyHeader<H>;
    type Future = FutureResult<H, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        match request.headers_mut().remove() {
            Some(h) => ok(h),
            None => err(Ok(EmptyHeader {
                _marker: PhantomData,
            })),
        }
    }
}

pub fn header_opt<H: header::Header, E>() -> HeaderOptional<H, E> {
    HeaderOptional {
        _marker: PhantomData,
    }
}

pub struct HeaderOptional<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Copy for HeaderOptional<H, E> {}

impl<H, E> Clone for HeaderOptional<H, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> fmt::Debug for HeaderOptional<H, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HeaderOpt").finish()
    }
}

impl<H: header::Header, E> Endpoint for HeaderOptional<H, E> {
    type Item = Option<H>;
    type Error = E;
    type Result = HeaderOptionalResult<H, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(HeaderOptionalResult {
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct HeaderOptionalResult<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H: header::Header, E> EndpointResult for HeaderOptionalResult<H, E> {
    type Item = Option<H>;
    type Error = E;
    type Future = FutureResult<Option<H>, Result<E, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        ok(request.headers_mut().remove())
    }
}

#[allow(missing_docs)]
pub struct EmptyHeader<H: header::Header> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: header::Header> fmt::Debug for EmptyHeader<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EmptyHeader").finish()
    }
}

impl<H: header::Header> fmt::Display for EmptyHeader<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The header '{}' is not given",
            <H as header::Header>::header_name()
        )
    }
}

impl<H: header::Header> Error for EmptyHeader<H> {
    fn description(&self) -> &str {
        "empty header"
    }
}

impl<H: header::Header> PartialEq for EmptyHeader<H> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<H: header::Header> IntoResponse for EmptyHeader<H> {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}
