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
use futures::future::{err, result, FutureResult};
use endpoint::{Endpoint, EndpointContext, EndpointResult, Request};
use http::{self, FromHeader};

#[allow(missing_docs)]
pub fn header<H: FromHeader>() -> Header<H> {
    Header {
        _marker: PhantomData,
    }
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
    type Error = HeaderError<H>;
    type Result = HeaderResult<H>;

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
pub struct HeaderResult<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: FromHeader> EndpointResult for HeaderResult<H> {
    type Item = H;
    type Error = HeaderError<H>;
    type Future = FutureResult<H, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let h = request
            .headers_mut()
            .get(<H as FromHeader>::header_name())
            .expect(&format!(
                "The value of header {} has already taken",
                H::header_name().as_str()
            ));
        result(H::from_header(&h).map_err(|e| Ok(HeaderError::Parsing(e))))
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
    type Error = HeaderError<H>;
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
    type Error = HeaderError<H>;
    type Future = FutureResult<H, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        match request.headers_mut().get(<H as FromHeader>::header_name()) {
            Some(h) => result(H::from_header(h).map_err(|e| Ok(HeaderError::Parsing(e)))),
            None => err(Ok(HeaderError::EmptyHeader)),
        }
    }
}

#[allow(missing_docs)]
pub fn header_opt<H: FromHeader>() -> HeaderOptional<H> {
    HeaderOptional {
        _marker: PhantomData,
    }
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
        f.debug_struct("HeaderOptional").finish()
    }
}

impl<H: FromHeader> Endpoint for HeaderOptional<H> {
    type Item = Option<H>;
    type Error = HeaderError<H>;
    type Result = HeaderOptionalResult<H>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(HeaderOptionalResult {
            _marker: PhantomData,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct HeaderOptionalResult<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H: FromHeader> EndpointResult for HeaderOptionalResult<H> {
    type Item = Option<H>;
    type Error = HeaderError<H>;
    type Future = FutureResult<Option<H>, Result<Self::Error, http::Error>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let h = request.headers_mut().get(<H as FromHeader>::header_name());
        let h = h.map_or(Ok(None), |h| {
            H::from_header(h)
                .map(Some)
                .map_err(|e| Ok(HeaderError::Parsing(e)))
        });
        result(h)
    }
}

#[allow(missing_docs)]
pub enum HeaderError<H: FromHeader> {
    EmptyHeader,
    Parsing(H::Error),
}

impl<H: FromHeader> fmt::Debug for HeaderError<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EmptyHeader").finish()
    }
}

impl<H: FromHeader> fmt::Display for HeaderError<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The header '{}' is not given",
            <H as FromHeader>::header_name().as_str()
        )
    }
}

impl<H: FromHeader> Error for HeaderError<H> {
    fn description(&self) -> &str {
        "empty header"
    }
}

impl<H: FromHeader> PartialEq for HeaderError<H> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
