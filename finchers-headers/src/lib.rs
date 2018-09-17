#![feature(rust_2018_preview)]
#![feature(tool_lints)] // for clippy

//! A collection of endpoints for parsing HTTP headers.

#![doc(
    html_root_url = "https://docs.rs/finchers-headers/0.1.0-alpha.1",
    test(attr(feature(rust_2018_preview))),
)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    unused,
)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "strict", doc(test(attr(deny(warnings)))))]

extern crate failure;
extern crate finchers;
extern crate futures;
extern crate http;
extern crate hyperx;

use finchers::endpoint::{Context, Endpoint, EndpointError, EndpointResult};
use finchers::error::Error;
use futures::future::{ready, Ready};
use std::fmt;
use std::marker::PhantomData;

use crate::header::TypedHeader;

/// Create an endpoint which parses a header field to the specified type.
pub fn header<T>() -> Header<T>
where
    T: TypedHeader,
{
    Header {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Header<T: TypedHeader> {
    _marker: PhantomData<T>,
}

impl<T: TypedHeader> Copy for Header<T> {}

impl<T: TypedHeader> Clone for Header<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: TypedHeader> fmt::Debug for Header<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Header").finish()
    }
}

impl<'a, T: TypedHeader + 'a> Endpoint<'a> for Header<T> {
    type Output = (T::Output,);
    type Future = Ready<Result<Self::Output, Error>>;

    fn apply(&self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match cx.input().headers().get(T::NAME) {
            Some(h) => T::parse_header(h)
                .map(|parsed| ready(Ok((parsed,))))
                .map_err(|err| EndpointError::custom(finchers::error::bad_request(err.into()))),
            None => Err(EndpointError::custom(finchers::error::bad_request(
                "missing authorization header",
            ))),
        }
    }
}

macro_rules! define_header_endpoints {
    ($($name:ident => $T:ty,)*) => {$(
        #[allow(missing_docs)]
        #[inline]
        pub fn $name() -> Header<$T> {
            header()
        }
    )*};
}

define_header_endpoints! {
    accept => crate::header::Accept,
    accept_charset => crate::header::AcceptCharset,
    accept_encoding => crate::header::AcceptEncoding,
    accept_language => crate::header::AcceptLanguage,
    accept_ranges => crate::header::AcceptRanges,
    connection => crate::header::Connection,
    content_encoding => crate::header::ContentEncoding,
    content_disposition => crate::header::ContentDisposition,
    content_language => crate::header::ContentLanguage,
    content_length => crate::header::ContentLength,
    content_type => crate::header::ContentType,
    date => crate::header::Date,
    expect => crate::header::Expect,
    host => crate::header::Host,
    if_match => crate::header::IfMatch,
    if_modified_since => crate::header::IfModifiedSince,
    if_none_match => crate::header::IfNoneMatch,
    if_range => crate::header::IfRange,
    if_unmodified_since => crate::header::IfUnmodifiedSince,
    origin => crate::header::Origin,
    referer => crate::header::Referer,
    te => crate::header::Te,
    user_agent => crate::header::UserAgent,
    warning => crate::header::Warning,
}

/// The definition of header types.
///
/// This module contains some re-exported types from `hyperx` crate.
pub mod header {
    pub use hyperx::header::{
        Accept, AcceptCharset, AcceptEncoding, AcceptLanguage, AcceptRanges, Charset, Connection,
        ConnectionOption, ContentDisposition, ContentEncoding, ContentLanguage, ContentLength,
        ContentType, Date, DispositionParam, DispositionType, Encoding, EntityTag, Expect, Host,
        HttpDate, IfMatch, IfModifiedSince, IfNoneMatch, IfRange, IfUnmodifiedSince, LanguageTag,
        Origin, Quality, QualityItem, RangeUnit, Referer, Te, UserAgent, Warning,
    };
    pub use hyperx::mime::Mime;

    use failure;
    use http;
    use http::header::{HeaderName, HeaderValue};
    use hyperx;

    /// Type alias represents a `Vec` which holds qualified items.
    pub type QualifiedVec<T> = Vec<QualityItem<T>>;

    /// A trait representing a pair of header field and value.
    pub trait TypedHeader {
        /// The name of header field associated with this type.
        #[allow(clippy::declare_interior_mutable_const)]
        const NAME: HeaderName;

        /// The value type of header field which will be returned from `parse_header`.
        type Output;

        /// The error type which will be returned from `parse_header`.
        type Error: Into<failure::Error>;

        /// Parse a header from a raw header value.
        fn parse_header(value: &HeaderValue) -> Result<Self::Output, Self::Error>;
    }

    macro_rules! impl_typed_headers {
        ($([$NAME:ident] $T:ty,)*) => {$(
            impl TypedHeader for $T {
                const NAME: HeaderName = http::header::$NAME;

                type Output = Self;
                type Error = hyperx::Error;

                fn parse_header(value: &HeaderValue) -> Result<Self::Output, Self::Error> {
                    <Self as hyperx::header::Header>::parse_header(&value.as_bytes().into())
                }
            }
        )*};

        ($([$NAME:ident] $T:ty => $O:ty,)*) => {$(
            impl TypedHeader for $T {
                const NAME: HeaderName = http::header::$NAME;

                type Output = $O;
                type Error = hyperx::Error;

                fn parse_header(value: &HeaderValue) -> Result<Self::Output, Self::Error> {
                    <Self as hyperx::header::Header>::parse_header(&value.as_bytes().into())
                        .map(|parsed| parsed.0)
                }
            }
        )*};
    }

    impl_typed_headers! {
        [ACCEPT] Accept => QualifiedVec<Mime>,
        [ACCEPT_CHARSET] AcceptCharset => QualifiedVec<Charset>,
        [ACCEPT_ENCODING] AcceptEncoding => QualifiedVec<Encoding>,
        [ACCEPT_LANGUAGE] AcceptLanguage => QualifiedVec<LanguageTag>,
        [ACCEPT_RANGES] AcceptRanges => Vec<RangeUnit>,
        [CONNECTION] Connection => Vec<ConnectionOption>,
        [CONTENT_ENCODING] ContentEncoding => Vec<Encoding>,
        [CONTENT_LANGUAGE] ContentLanguage => QualifiedVec<LanguageTag>,
        [CONTENT_LENGTH] ContentLength => u64,
        [CONTENT_TYPE] ContentType => Mime,
        [DATE] Date => HttpDate,
        [IF_MODIFIED_SINCE] IfModifiedSince => HttpDate,
        [IF_UNMODIFIED_SINCE] IfUnmodifiedSince => HttpDate,
        [TE] Te => QualifiedVec<Encoding>,
    }

    impl_typed_headers! {
        [CONTENT_DISPOSITION] ContentDisposition,
        [EXPECT] Expect,
        [HOST] Host,
        [IF_MATCH] IfMatch,
        [IF_NONE_MATCH] IfNoneMatch,
        [IF_RANGE] IfRange,
        [ORIGIN] Origin,
        [REFERER] Referer,
        [USER_AGENT] UserAgent,
        [WARNING] Warning,
    }
}

/// Specialized endpoints for parsing `Authorization` headers.
pub mod authorization {
    use failure::err_msg;
    use futures::future::{ready, Ready};
    use hyperx::header;
    use hyperx::header::Header;
    use std::fmt;
    use std::marker::PhantomData;

    use finchers::endpoint::{Context, Endpoint, EndpointError, EndpointResult};
    use finchers::error::Error;

    pub use hyperx::header::{Basic, Bearer, Scheme};

    #[allow(missing_docs)]
    pub fn basic() -> Authorization<Basic> {
        custom()
    }

    #[allow(missing_docs)]
    pub fn bearer() -> Authorization<Bearer> {
        custom()
    }

    #[allow(missing_docs)]
    pub fn custom<S: Scheme>() -> Authorization<S> {
        Authorization {
            _marker: PhantomData,
        }
    }

    #[allow(missing_docs)]
    pub struct Authorization<S: Scheme> {
        _marker: PhantomData<S>,
    }

    impl<S: Scheme> Copy for Authorization<S> {}

    impl<S: Scheme> Clone for Authorization<S> {
        #[inline]
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<S: Scheme> fmt::Debug for Authorization<S> {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.debug_struct("Authorization").finish()
        }
    }

    impl<'a, S: Scheme + 'static> Endpoint<'a> for Authorization<S> {
        type Output = (S,);
        type Future = Ready<Result<Self::Output, Error>>;

        fn apply(&self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
            match cx.input().headers().get("authorization") {
                Some(h) => header::Authorization::<S>::parse_header(&h.as_bytes().into())
                    .map(|header::Authorization(scheme)| ready(Ok((scheme,))))
                    .map_err(|err| EndpointError::custom(finchers::error::bad_request(err))),
                None => Err(EndpointError::custom(err_msg(
                    "missing authorization header",
                ))),
            }
        }
    }

    #[allow(missing_docs)]
    pub fn proxy<S: Scheme>() -> ProxyAuthorization<S> {
        ProxyAuthorization {
            _marker: PhantomData,
        }
    }

    #[allow(missing_docs)]
    pub struct ProxyAuthorization<S: Scheme> {
        _marker: PhantomData<S>,
    }

    impl<S: Scheme> Copy for ProxyAuthorization<S> {}

    impl<S: Scheme> Clone for ProxyAuthorization<S> {
        #[inline]
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<S: Scheme> fmt::Debug for ProxyAuthorization<S> {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.debug_struct("Authorization").finish()
        }
    }

    impl<'a, S: Scheme + 'static> Endpoint<'a> for ProxyAuthorization<S> {
        type Output = (S,);
        type Future = Ready<Result<Self::Output, Error>>;

        fn apply(&self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
            match cx.input().headers().get("proxy-authorization") {
                Some(h) => header::Authorization::<S>::parse_header(&h.as_bytes().into())
                    .map(|header::Authorization(scheme)| ready(Ok((scheme,))))
                    .map_err(|err| EndpointError::custom(finchers::error::bad_request(err))),
                None => Err(EndpointError::custom(err_msg(
                    "missing authorization header",
                ))),
            }
        }
    }
}
