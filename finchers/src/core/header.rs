#![allow(missing_docs)]

use std::any::Any;
use std::error::Error;
use std::str::FromStr;
use hyper;
use hyper::header::{self, Header};
use super::NeverReturn;

pub trait FromHeader: 'static + Sized {
    type Error: Error + 'static;

    fn header_name() -> &'static str;

    fn from_header(s: &[u8]) -> Result<Self, Self::Error>;
}

impl<H: FromHeader> FromHeader for Option<H> {
    type Error = NeverReturn;

    #[inline]
    fn header_name() -> &'static str {
        H::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        Ok(H::from_header(s).ok())
    }
}

impl<H: FromHeader> FromHeader for Result<H, H::Error> {
    type Error = NeverReturn;

    #[inline]
    fn header_name() -> &'static str {
        H::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        Ok(H::from_header(s))
    }
}

macro_rules! impl_for_hyper_headers {
    ($($t:ty;)*) => {$(
        impl FromHeader for $t {
            type Error = hyper::Error;

            #[inline]
            fn header_name() -> &'static str {
                <$t as Header>::header_name()
            }

            #[inline]
            fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
                <$t as Header>::parse_header(&s.into())
            }
        }
    )*};
}

impl_for_hyper_headers! {
    header::AcceptCharset;
    header::AcceptEncoding;
    header::AcceptLanguage;
    header::AcceptRanges;
    header::Accept;
    header::AccessControlAllowCredentials;
    header::AccessControlAllowHeaders;
    header::AccessControlAllowMethods;
    header::AccessControlAllowOrigin;
    header::AccessControlExposeHeaders;
    header::AccessControlMaxAge;
    header::AccessControlRequestHeaders;
    header::AccessControlRequestMethod;
    header::Allow;
    header::CacheControl;
    header::Connection;
    header::ContentDisposition;
    header::ContentEncoding;
    header::ContentLanguage;
    header::ContentLength;
    header::ContentLocation;
    header::ContentRange;
    header::ContentType;
    header::Cookie;
    header::Date;
    header::ETag;
    header::Expect;
    header::Expires;
    header::Host;
    header::IfMatch;
    header::IfModifiedSince;
    header::IfNoneMatch;
    header::IfRange;
    header::IfUnmodifiedSince;
    header::LastEventId;
    header::LastModified;
    header::Link;
    header::Location;
    header::Origin;
    header::Pragma;
    header::Prefer;
    header::PreferenceApplied;
    header::Range;
    header::Referer;
    header::ReferrerPolicy;
    header::RetryAfter;
    header::Server;
    header::SetCookie;
    header::StrictTransportSecurity;
    header::Te;
    header::TransferEncoding;
    header::Upgrade;
    header::UserAgent;
    header::Vary;
    header::Warning;
}

// TODO: From

impl<S> FromHeader for header::Authorization<S>
where
    S: header::Scheme + Any,
    <S as FromStr>::Err: 'static,
{
    type Error = hyper::Error;

    #[inline]
    fn header_name() -> &'static str {
        <Self as Header>::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        <Self as Header>::parse_header(&s.into())
    }
}

impl<S> FromHeader for header::ProxyAuthorization<S>
where
    S: header::Scheme + Any,
    <S as FromStr>::Err: 'static,
{
    type Error = hyper::Error;

    #[inline]
    fn header_name() -> &'static str {
        <Self as Header>::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        <Self as Header>::parse_header(&s.into())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub struct HyperHeader<H>(pub H);

impl<H: Header + Clone> FromHeader for HyperHeader<H> {
    type Error = hyper::Error;

    #[inline]
    fn header_name() -> &'static str {
        <H as Header>::header_name()
    }

    #[inline]
    fn from_header(s: &[u8]) -> Result<Self, Self::Error> {
        <H as Header>::parse_header(&s.into()).map(HyperHeader)
    }
}
