//! A set of components which provides the support for CORS in Finchers.

// master
#![doc(html_root_url = "https://finchers-rs.github.io/finchers-cors")]
// released
//#![doc(html_root_url = "https://docs.rs/finchers-cors/0.1.0-alpha.1")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    unused,
)]
//#![warn(rust_2018_compatibility)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "strict", doc(test(attr(deny(warnings)))))]

extern crate either;
#[macro_use]
extern crate failure;
extern crate finchers;
extern crate futures; // 0.1
extern crate http;

use std::collections::HashSet;
use std::time::Duration;

use finchers::endpoint::with_get_cx;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint, Wrapper};
use finchers::error::{Error, HttpError};
use finchers::input::Input;
use finchers::output::{Output, OutputContext};

use futures::{Async, Future, Poll};

use either::Either;
use http::header;
use http::header::{HeaderName, HeaderValue};
use http::{Method, Response, StatusCode, Uri};

/// A `Wrapper` for building an endpoint with CORS.
#[derive(Debug, Default)]
pub struct CorsFilter {
    origins: Option<HashSet<Uri>>,
    methods: Option<HashSet<Method>>,
    headers: Option<HashSet<HeaderName>>,
    max_age: Option<Duration>,
    allow_credentials: bool,
}

impl CorsFilter {
    /// Creates a `CorsFilter` with the default configuration.
    pub fn new() -> CorsFilter {
        Default::default()
    }

    #[allow(missing_docs)]
    pub fn allow_origin(mut self, origin: impl Into<Uri>) -> CorsFilter {
        self.origins
            .get_or_insert_with(Default::default)
            .insert(origin.into());
        self
    }

    #[allow(missing_docs)]
    pub fn allow_origins(mut self, origins: impl IntoIterator<Item = Uri>) -> CorsFilter {
        self.origins
            .get_or_insert_with(Default::default)
            .extend(origins);
        self
    }

    #[allow(missing_docs)]
    pub fn allow_method(mut self, method: Method) -> CorsFilter {
        self.methods
            .get_or_insert_with(Default::default)
            .insert(method);
        self
    }

    #[allow(missing_docs)]
    pub fn allow_methods(mut self, methods: impl IntoIterator<Item = Method>) -> CorsFilter {
        self.methods
            .get_or_insert_with(Default::default)
            .extend(methods);
        self
    }

    #[allow(missing_docs)]
    pub fn allow_header(mut self, header: HeaderName) -> CorsFilter {
        self.headers
            .get_or_insert_with(Default::default)
            .insert(header);
        self
    }

    #[allow(missing_docs)]
    pub fn allow_headers(mut self, headers: impl IntoIterator<Item = HeaderName>) -> CorsFilter {
        self.headers
            .get_or_insert_with(Default::default)
            .extend(headers);
        self
    }

    #[allow(missing_docs)]
    pub fn allow_credentials(self, enabled: bool) -> CorsFilter {
        CorsFilter {
            allow_credentials: enabled,
            ..self
        }
    }

    #[allow(missing_docs)]
    pub fn max_age(self, max_age: Duration) -> CorsFilter {
        CorsFilter {
            max_age: Some(max_age),
            ..self
        }
    }
}

impl<'a, E: Endpoint<'a>> Wrapper<'a, E> for CorsFilter {
    type Output = (CorsResponse<E::Output>,);
    type Endpoint = CorsEndpoint<E>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        let methods = self.methods.unwrap_or_else(|| {
            vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::HEAD,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ].into_iter()
            .collect()
        });

        let methods_value = HeaderValue::from_shared(
            methods
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, m)| {
                    if i > 0 {
                        acc += ",";
                    }
                    acc += m.as_str();
                    acc
                }).into(),
        ).expect("should be a valid header value");

        let headers_value = self.headers.as_ref().map(|hdrs| {
            HeaderValue::from_shared(
                hdrs.iter()
                    .enumerate()
                    .fold(String::new(), |mut acc, (i, hdr)| {
                        if i > 0 {
                            acc += ",";
                        }
                        acc += hdr.as_str();
                        acc
                    }).into(),
            ).expect("should be a valid header value")
        });

        CorsEndpoint {
            endpoint,
            origins: self.origins,
            methods,
            methods_value,
            headers: self.headers,
            headers_value,
            max_age: self.max_age,
            allow_credentials: self.allow_credentials,
        }
    }
}

/// An endpoint which represents a route with CORS handling.
///
/// The value of this type is generated by `CorsFilter::wrap()`.
#[derive(Debug)]
pub struct CorsEndpoint<E> {
    endpoint: E,
    origins: Option<HashSet<Uri>>,
    methods: HashSet<Method>,
    methods_value: HeaderValue,
    headers: Option<HashSet<HeaderName>>,
    headers_value: Option<HeaderValue>,
    max_age: Option<Duration>,
    allow_credentials: bool,
}

fn parse_origin(h: &HeaderValue) -> Result<Uri, CorsError> {
    let h_str = h.to_str().map_err(|_| CorsErrorKind::InvalidOrigin)?;
    let origin_uri: Uri = h_str.parse().map_err(|_| CorsErrorKind::InvalidOrigin)?;

    if origin_uri.scheme_part().is_none() {
        return Err(CorsErrorKind::InvalidOrigin.into());
    }

    if origin_uri.host().is_none() {
        return Err(CorsErrorKind::InvalidOrigin.into());
    }

    Ok(origin_uri)
}

impl<E> CorsEndpoint<E> {
    fn validate_origin_header(&self, input: &Input) -> Result<AllowedOrigin, CorsError> {
        let origin = input
            .headers()
            .get(header::ORIGIN)
            .ok_or_else(|| CorsErrorKind::MissingOrigin)?;
        let parsed_origin = parse_origin(origin)?;

        if let Some(ref origins) = self.origins {
            if !origins.contains(&parsed_origin) {
                return Err(CorsErrorKind::DisallowedOrigin.into());
            }
            return Ok(AllowedOrigin::Some(origin.clone()));
        }

        if self.allow_credentials {
            Ok(AllowedOrigin::Some(origin.clone()))
        } else {
            Ok(AllowedOrigin::Any)
        }
    }

    fn validate_request_method(&self, input: &Input) -> Result<Option<HeaderValue>, CorsError> {
        match input.headers().get(header::ACCESS_CONTROL_REQUEST_METHOD) {
            Some(h) => {
                let method: Method = h
                    .to_str()
                    .map_err(|_| CorsErrorKind::InvalidRequestMethod)?
                    .parse()
                    .map_err(|_| CorsErrorKind::InvalidRequestMethod)?;
                if self.methods.contains(&method) {
                    Ok(Some(self.methods_value.clone()))
                } else {
                    Err(CorsErrorKind::DisallowedRequestMethod.into())
                }
            }
            None => Ok(None),
        }
    }

    fn validate_request_headers(&self, input: &Input) -> Result<Option<HeaderValue>, CorsError> {
        match input.headers().get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
            Some(hdrs) => match self.headers {
                Some(ref headers) => {
                    let mut request_headers = HashSet::new();
                    let hdrs_str = hdrs
                        .to_str()
                        .map_err(|_| CorsErrorKind::InvalidRequestHeaders)?;
                    for hdr in hdrs_str.split(',').map(|s| s.trim()) {
                        let hdr: HeaderName = hdr
                            .parse()
                            .map_err(|_| CorsErrorKind::InvalidRequestHeaders)?;
                        request_headers.insert(hdr);
                    }

                    if !headers.is_superset(&request_headers) {
                        return Err(CorsErrorKind::DisallowedRequestHeaders.into());
                    }

                    Ok(self.headers_value.clone())
                }
                None => Ok(Some(hdrs.clone())),
            },
            None => Ok(None),
        }
    }

    fn handle_preflight_request(
        &self,
        input: &Input,
    ) -> Result<Either<Response<()>, AllowedOrigin>, CorsError> {
        let origin = self.validate_origin_header(input)?;
        match *input.method() {
            Method::OPTIONS => match self.validate_request_method(input)? {
                Some(allow_methods) => {
                    let allow_headers = self.validate_request_headers(input)?;

                    let mut response = Response::new(());
                    response
                        .headers_mut()
                        .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.into());
                    response
                        .headers_mut()
                        .insert(header::ACCESS_CONTROL_ALLOW_METHODS, allow_methods);

                    if let Some(allow_headers) = allow_headers {
                        response
                            .headers_mut()
                            .insert(header::ACCESS_CONTROL_ALLOW_HEADERS, allow_headers);
                    }

                    if let Some(max_age) = self.max_age {
                        response
                            .headers_mut()
                            .insert(header::ACCESS_CONTROL_MAX_AGE, max_age.as_secs().into());
                    }

                    Ok(Either::Left(response))
                }
                None => Ok(Either::Right(origin)),
            },
            ref method if self.methods.contains(method) => Ok(Either::Right(origin)),
            _ => Err(CorsErrorKind::DisallowedRequestMethod.into()),
        }
    }
}

impl<'a, E> Endpoint<'a> for CorsEndpoint<E>
where
    E: Endpoint<'a>,
{
    type Output = (CorsResponse<E::Output>,);
    type Future = CorsFuture<'a, E>;

    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(CorsFuture {
            future: self.endpoint.apply(cx)?,
            endpoint: self,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct CorsFuture<'a, E: Endpoint<'a>> {
    future: E::Future,
    endpoint: &'a CorsEndpoint<E>,
}

impl<'a, E> Future for CorsFuture<'a, E>
where
    E: Endpoint<'a>,
{
    type Item = (CorsResponse<E::Output>,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let endpoint = self.endpoint;

        match { with_get_cx(|input| endpoint.handle_preflight_request(&*input))? } {
            Either::Left(response) => Ok(Async::Ready((CorsResponse(
                CorsResponseKind::Preflight(response),
            ),))),
            Either::Right(allowed_origin) => {
                with_get_cx(|input| {
                    input
                        .response_headers()
                        .append(header::ACCESS_CONTROL_ALLOW_ORIGIN, allowed_origin.into());
                    if endpoint.allow_credentials {
                        input.response_headers().append(
                            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                            HeaderValue::from_static("true"),
                        );
                    }
                });
                self.future
                    .poll()
                    .map(|x| x.map(|output| (CorsResponse(CorsResponseKind::Normal(output)),)))
            }
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct CorsResponse<T>(CorsResponseKind<T>);

#[derive(Debug)]
enum CorsResponseKind<T> {
    Preflight(Response<()>),
    Normal(T),
}

impl<T: Output> Output for CorsResponse<T> {
    type Body = Option<T::Body>;
    type Error = T::Error;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        match self.0 {
            CorsResponseKind::Preflight(response) => Ok(response.map(|_| None)),
            CorsResponseKind::Normal(normal) => {
                normal.respond(cx).map(|response| response.map(Some))
            }
        }
    }
}

#[derive(Debug, Clone)]
enum AllowedOrigin {
    Some(HeaderValue),
    Any,
}

impl Into<HeaderValue> for AllowedOrigin {
    fn into(self) -> HeaderValue {
        match self {
            AllowedOrigin::Some(v) => v,
            AllowedOrigin::Any => HeaderValue::from_static("*"),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[fail(display = "Invalid CORS request: {}", kind)]
pub struct CorsError {
    kind: CorsErrorKind,
}

impl From<CorsErrorKind> for CorsError {
    fn from(kind: CorsErrorKind) -> CorsError {
        CorsError { kind }
    }
}

impl HttpError for CorsError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl CorsError {
    #[allow(missing_docs)]
    pub fn kind(&self) -> &CorsErrorKind {
        &self.kind
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
pub enum CorsErrorKind {
    #[fail(display = "the Origin is missing.")]
    MissingOrigin,

    #[fail(display = "the provided Origin is not a valid value.")]
    InvalidOrigin,

    #[fail(display = "the provided Origin is not allowed.")]
    DisallowedOrigin,

    #[fail(display = "the provided Access-Control-Request-Method is not a valid value.")]
    InvalidRequestMethod,

    #[fail(display = "the provided Access-Control-Request-Method is not allowed.")]
    DisallowedRequestMethod,

    #[fail(display = "the provided Access-Control-Request-Headers is not a valid value.")]
    InvalidRequestHeaders,

    #[fail(display = "the provided Access-Control-Request-Headers is not allowed.")]
    DisallowedRequestHeaders,
}
