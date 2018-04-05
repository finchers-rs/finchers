//! Components for accessing of HTTP headers
//!
//! There are three endpoint for accessing the value of HTTP header:
//!
//! * `Header<H, E>` - Returns the value of `H` from the header map. If the value of `H` is not found, then skipping the current route.
//! * `HeaderRequired<H>` - Similar to `Header`, but always matches and returns an error if `H` is not found.
//! * `HeaderOptional<H, E>` - Similar to `Header`, but always matches and returns a `None` if `H` is not found.

use finchers_core::error::{BadRequest, Error, NotPresent};
use finchers_core::request::{with_input, FromHeader, Input};
use futures::{Future, Poll};
use std::fmt;
use std::marker::PhantomData;
use {Context, Endpoint};

#[allow(missing_docs)]
pub fn header<H: FromHeader>() -> Header<H> {
    Header { _marker: PhantomData }
}

#[allow(missing_docs)]
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

impl<H: FromHeader> Endpoint for Header<H> {
    type Item = H;
    type Future = HeaderFuture<H>;

    fn apply(&self, input: &Input, _: &mut Context) -> Option<Self::Future> {
        if input.headers().contains_key(H::header_name()) {
            Some(HeaderFuture { _marker: PhantomData })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct HeaderFuture<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: FromHeader> Future for HeaderFuture<H> {
    type Item = H;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        with_input(|input| {
            let value = input
                .headers()
                .get(H::header_name())
                .expect(&format!("The value of header {} has already taken", H::header_name()));
            H::from_header(value.as_bytes())
                .map(Into::into)
                .map_err(|e| BadRequest::new(e).into())
        })
    }
}

#[allow(missing_docs)]
pub fn header_req<H: FromHeader>() -> HeaderRequired<H> {
    HeaderRequired { _marker: PhantomData }
}

#[allow(missing_docs)]
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

impl<H: FromHeader> Endpoint for HeaderRequired<H> {
    type Item = H;
    type Future = HeaderRequiredFuture<H>;

    fn apply(&self, _: &Input, _: &mut Context) -> Option<Self::Future> {
        Some(HeaderRequiredFuture { _marker: PhantomData })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct HeaderRequiredFuture<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: FromHeader> Future for HeaderRequiredFuture<H> {
    type Item = H;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        with_input(|input| match input.headers().get(H::header_name()) {
            Some(h) => H::from_header(h.as_bytes())
                .map(Into::into)
                .map_err(|e| BadRequest::new(e).into()),
            None => Err(NotPresent::new(format!(
                "The header `{}' does not exist in the request",
                H::header_name()
            )).into()),
        })
    }
}

#[allow(missing_docs)]
pub fn header_opt<H: FromHeader>() -> HeaderOptional<H> {
    HeaderOptional { _marker: PhantomData }
}

#[allow(missing_docs)]
pub struct HeaderOptional<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for HeaderOptional<H> {}

impl<H> Clone for HeaderOptional<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for HeaderOptional<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HeaderOpt").finish()
    }
}

impl<H: FromHeader> Endpoint for HeaderOptional<H> {
    type Item = Option<H>;
    type Future = HeaderOptionalFuture<H>;

    fn apply(&self, _: &Input, _: &mut Context) -> Option<Self::Future> {
        Some(HeaderOptionalFuture { _marker: PhantomData })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct HeaderOptionalFuture<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: FromHeader> Future for HeaderOptionalFuture<H> {
    type Item = Option<H>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        with_input(|input| {
            Ok(input
                .headers()
                .get(H::header_name())
                .and_then(|h| H::from_header(h.as_bytes()).ok())
                .into())
        })
    }
}
