//! Components for accessing of HTTP headers
//!
//! There are three endpoint for accessing the value of HTTP header:
//!
//! * `Header<H, E>` - Returns the value of `H` from the header map. If the value of `H` is not found, then skipping the current route.
//! * `HeaderRequired<H>` - Similar to `Header`, but always matches and returns an error if `H` is not found.
//! * `HeaderOptional<H, E>` - Similar to `Header`, but always matches and returns a `None` if `H` is not found.

use std::fmt;
use std::error::Error;
use std::marker::PhantomData;
use futures::future::{err, ok, FutureResult};
use endpoint::{Endpoint, EndpointContext, EndpointResult};
use errors::StdErrorResponseBuilder;
use http::{self, Header as HeaderTrait, IntoResponse, Request, Response};

#[allow(missing_docs)]
pub fn header<H: HeaderTrait, E>() -> Header<H, E> {
    Header {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
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

impl<H: HeaderTrait, E> Endpoint for Header<H, E> {
    type Item = H;
    type Error = E;
    type Result = HeaderResult<H, E>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        if ctx.headers()
            .contains_key(<H as HeaderTrait>::header_name())
        {
            Some(HeaderResult {
                _marker: PhantomData,
            })
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct HeaderResult<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H: HeaderTrait, E> EndpointResult for HeaderResult<H, E> {
    type Item = H;
    type Error = E;
    type Future = FutureResult<H, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        // TODO: appropriate error handling
        let h = request
            .headers_mut()
            .remove(<H as HeaderTrait>::header_name())
            .expect(&format!(
                "The value of header {} has already taken",
                H::header_name()
            ));
        let h = H::parse_header(&h.to_str().unwrap().into()).unwrap();
        ok(h)
    }
}

#[allow(missing_docs)]
pub fn header_req<H: HeaderTrait>() -> HeaderRequired<H> {
    HeaderRequired {
        _marker: PhantomData,
    }
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

impl<H: HeaderTrait> Endpoint for HeaderRequired<H> {
    type Item = H;
    type Error = EmptyHeader<H>;
    type Result = HeaderRequiredResult<H>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(HeaderRequiredResult {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct HeaderRequiredResult<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: HeaderTrait> EndpointResult for HeaderRequiredResult<H> {
    type Item = H;
    type Error = EmptyHeader<H>;
    type Future = FutureResult<H, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        match request
            .headers_mut()
            .remove(<H as HeaderTrait>::header_name())
        {
            Some(h) => {
                let h = H::parse_header(&h.to_str().unwrap().into()).unwrap();
                ok(h)
            }
            None => err(Ok(EmptyHeader {
                _marker: PhantomData,
            })),
        }
    }
}

#[allow(missing_docs)]
pub fn header_opt<H: HeaderTrait, E>() -> HeaderOptional<H, E> {
    HeaderOptional {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
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

impl<H: HeaderTrait, E> Endpoint for HeaderOptional<H, E> {
    type Item = Option<H>;
    type Error = E;
    type Result = HeaderOptionalResult<H, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(HeaderOptionalResult {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct HeaderOptionalResult<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H: HeaderTrait, E> EndpointResult for HeaderOptionalResult<H, E> {
    type Item = Option<H>;
    type Error = E;
    type Future = FutureResult<Option<H>, Result<E, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let h = request
            .headers_mut()
            .remove(<H as HeaderTrait>::header_name());
        let h = h.map(|h| H::parse_header(&h.to_str().unwrap().into()).unwrap());
        ok(h)
    }
}

#[allow(missing_docs)]
pub struct EmptyHeader<H: HeaderTrait> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: HeaderTrait> fmt::Debug for EmptyHeader<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EmptyHeader").finish()
    }
}

impl<H: HeaderTrait> fmt::Display for EmptyHeader<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The header '{}' is not given",
            <H as HeaderTrait>::header_name()
        )
    }
}

impl<H: HeaderTrait> Error for EmptyHeader<H> {
    fn description(&self) -> &str {
        "empty header"
    }
}

impl<H: HeaderTrait> PartialEq for EmptyHeader<H> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<H: HeaderTrait> IntoResponse for EmptyHeader<H> {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}
