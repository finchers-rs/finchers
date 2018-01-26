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
use http::{self, FromHeader, IntoResponse, Request, Response};

#[allow(missing_docs)]
pub fn header<H: FromHeader, E>() -> Header<H, E> {
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

impl<H: FromHeader, E> Endpoint for Header<H, E> {
    type Item = H;
    type Error = E;
    type Result = HeaderResult<H, E>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        if ctx.headers().contains_key(<H as FromHeader>::header_name()) {
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

impl<H: FromHeader, E> EndpointResult for HeaderResult<H, E> {
    type Item = H;
    type Error = E;
    type Future = FutureResult<H, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        // TODO: appropriate error handling
        let h = request
            .headers_mut()
            .get(<H as FromHeader>::header_name())
            .expect(&format!(
                "The value of header {} has already taken",
                H::header_name().as_str()
            ));
        let h = match H::from_header(&h) {
            Ok(h) => h,
            Err(..) => panic!(),
        };
        ok(h)
    }
}

#[allow(missing_docs)]
pub fn header_req<H: FromHeader>() -> HeaderRequired<H> {
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

impl<H: FromHeader> Endpoint for HeaderRequired<H> {
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

impl<H: FromHeader> EndpointResult for HeaderRequiredResult<H> {
    type Item = H;
    type Error = EmptyHeader<H>;
    type Future = FutureResult<H, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        match request.headers_mut().get(<H as FromHeader>::header_name()) {
            Some(h) => {
                let h = match H::from_header(h) {
                    Ok(h) => h,
                    Err(..) => panic!(),
                };
                ok(h)
            }
            None => err(Ok(EmptyHeader {
                _marker: PhantomData,
            })),
        }
    }
}

#[allow(missing_docs)]
pub fn header_opt<H: FromHeader, E>() -> HeaderOptional<H, E> {
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

impl<H: FromHeader, E> Endpoint for HeaderOptional<H, E> {
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

impl<H: FromHeader, E> EndpointResult for HeaderOptionalResult<H, E> {
    type Item = Option<H>;
    type Error = E;
    type Future = FutureResult<Option<H>, Result<E, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let h = request.headers_mut().get(<H as FromHeader>::header_name());
        let h = h.map(|h| match H::from_header(h) {
            Ok(h) => h,
            Err(..) => panic!(),
        });
        ok(h)
    }
}

#[allow(missing_docs)]
pub struct EmptyHeader<H: FromHeader> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: FromHeader> fmt::Debug for EmptyHeader<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EmptyHeader").finish()
    }
}

impl<H: FromHeader> fmt::Display for EmptyHeader<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The header '{}' is not given",
            <H as FromHeader>::header_name().as_str()
        )
    }
}

impl<H: FromHeader> Error for EmptyHeader<H> {
    fn description(&self) -> &str {
        "empty header"
    }
}

impl<H: FromHeader> PartialEq for EmptyHeader<H> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<H: FromHeader> IntoResponse for EmptyHeader<H> {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self).finish()
    }
}
