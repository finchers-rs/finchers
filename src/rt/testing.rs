//! The basic facilities for testing endpoints.

use std::borrow::Cow;
use std::io;
use std::str;

use bytes::{Buf, Bytes};
use futures::{future, stream, Async, Future, Stream};
use http;
use http::header;
use http::header::HeaderMap;
use http::{Request, Response};
use hyper::body::{Body, Payload};
use mime;
use mime::Mime;
use tokio::runtime::current_thread::Runtime;

use endpoint::Endpoint;
use error::Error;
use input::ReqBody;
use output;
use output::Output;

use super::app::app_service::{AppFuture, AppService};
use super::blocking::{with_set_runtime_mode, RuntimeMode};

struct AnnotatedRuntime<'a>(&'a mut Runtime);

impl<'a> AnnotatedRuntime<'a> {
    fn block_on<F: Future>(&mut self, mut future: F) -> Result<F::Item, F::Error> {
        self.0.block_on(future::poll_fn(move || {
            with_set_runtime_mode(RuntimeMode::CurrentThread, || future.poll())
        }))
    }
}

#[allow(missing_docs)]
pub fn runner<E>(endpoint: E) -> TestRunner<E>
where
    for<'a> E: Endpoint<'a>,
{
    TestRunner::new(endpoint).expect("failed to start the runtime")
}

/// A test runner for emulating the behavior of endpoints in the server.
///
/// It uses internally the current thread version of Tokio runtime for executing
/// asynchronous processes.
#[derive(Debug)]
pub struct TestRunner<E> {
    endpoint: E,
    headers: Option<HeaderMap>,
    rt: Runtime,
}

impl<E> TestRunner<E> {
    /// Create a `TestRunner` from the specified endpoint.
    pub fn new(endpoint: E) -> io::Result<TestRunner<E>>
    where
        for<'e> E: Endpoint<'e>,
    {
        Ok(TestRunner {
            endpoint,
            headers: None,
            rt: Runtime::new()?,
        })
    }

    /// Create a `TestRunner` from the specified endpoint with a Tokio runtime.
    pub fn with_runtime(endpoint: E, rt: Runtime) -> TestRunner<E> {
        TestRunner {
            endpoint,
            rt,
            headers: None,
        }
    }

    /// Returns a reference to the header map, whose values are set before
    /// applying the request to endpoint.
    pub fn default_headers(&mut self) -> &mut HeaderMap {
        self.headers.get_or_insert_with(Default::default)
    }

    fn apply_inner<'a, Req, F, R>(&'a mut self, request: Req, f: F) -> R
    where
        E: Endpoint<'a>,
        Req: RequestData,
        F: FnOnce(AppFuture<'a, E>, &mut AnnotatedRuntime<'_>) -> R,
    {
        let (mut parts, body) = request
            .into_request()
            .expect("failed to construct a request")
            .into_parts();
        if let Some(headers) = self.headers.clone() {
            parts.headers.extend(headers);
        }
        if let Some(mime) = body.content_type() {
            parts
                .headers
                .entry(header::CONTENT_TYPE)
                .unwrap()
                .or_insert(
                    mime.as_ref()
                        .parse()
                        .expect("should be a valid header value"),
                );
        }
        let body = body.into_req_body();
        if let Some(len) = body.content_length() {
            parts
                .headers
                .entry(header::CONTENT_LENGTH)
                .unwrap()
                .or_insert(
                    len.to_string()
                        .parse()
                        .expect("should be a valid header value"),
                );
        }

        let request = Request::from_parts(parts, body);
        let future = AppService::new(&self.endpoint).dispatch(request);

        f(future, &mut AnnotatedRuntime(&mut self.rt))
    }

    /// Applys the given request to the inner endpoint and retrieves the result of returned future.
    pub fn apply_raw<'a, Req>(&'a mut self, request: Req) -> Result<E::Output, Error>
    where
        E: Endpoint<'a>,
        Req: RequestData,
    {
        self.apply_inner(request, |mut future, rt| {
            rt.block_on(future::poll_fn(|| future.poll_endpoint()))
        })
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn apply<'a, T, Req>(&'a mut self, request: Req) -> Result<T, Error>
    where
        E: Endpoint<'a, Output = (T,)>,
        Req: RequestData,
    {
        self.apply_raw(request).map(|(x,)| x)
    }

    /// Retrieves the retrieves the result of future returned from `Endpoint::apply`,
    /// and converting it into an HTTP response by calling `Output::respond`.
    pub fn apply_output<'a, Req, Bd>(&'a mut self, request: Req) -> Result<Response<Bd>, Error>
    where
        E: Endpoint<'a>,
        E::Output: Output<Body = Bd>,
        Req: RequestData,
        Bd: output::body::ResBody,
    {
        self.apply_inner(request, |mut future, rt| {
            rt.block_on(future::poll_fn(|| future.poll_output()))
        })
    }

    /// Gets the response of specified HTTP request.
    pub fn apply_all<'a, Req>(&'a mut self, request: Req) -> Response<ResBody>
    where
        E: Endpoint<'a>,
        E::Output: Output,
        Req: RequestData,
    {
        self.apply_inner(request, |mut future, rt| {
            let response = rt.block_on(future::poll_fn(|| future.poll())).unwrap();
            let (parts, mut payload) = response.into_parts();

            // construct ResBody
            let content_length = payload.content_length();

            let data = rt
                .block_on(
                    stream::poll_fn(|| match payload.poll_data() {
                        Ok(Async::Ready(data)) => Ok(Async::Ready(data.map(Buf::collect))),
                        Ok(Async::NotReady) => Ok(Async::NotReady),
                        Err(err) => Err(err),
                    }).collect(),
                ).expect("error during sending the response body.");

            let trailers = rt
                .block_on(future::poll_fn(|| payload.poll_trailers()))
                .expect("error during sending trailers.");

            let body = ResBody {
                data,
                trailers,
                content_length,
            };

            Response::from_parts(parts, body)
        })
    }

    /// Returns a reference to the underlying Tokio runtime.
    pub fn runtime(&mut self) -> &mut Runtime {
        &mut self.rt
    }
}

/// A trait representing a request used by the test runner.
#[allow(missing_docs)]
pub trait RequestData {
    type Body: RequestBody;

    fn into_request(self) -> http::Result<Request<Self::Body>>;
}

impl<'a> RequestData for &'a str {
    type Body = ();

    fn into_request(self) -> http::Result<Request<Self::Body>> {
        Request::get(self).body(())
    }
}

impl RequestData for String {
    type Body = ();

    fn into_request(self) -> http::Result<Request<Self::Body>> {
        Request::get(self).body(())
    }
}

impl RequestData for http::request::Builder {
    type Body = ();

    fn into_request(mut self) -> http::Result<Request<Self::Body>> {
        self.body(())
    }
}

impl<'a> RequestData for &'a mut http::request::Builder {
    type Body = ();

    fn into_request(self) -> http::Result<Request<Self::Body>> {
        self.body(())
    }
}

impl<T: RequestBody> RequestData for Request<T> {
    type Body = T;

    fn into_request(self) -> http::Result<Request<Self::Body>> {
        Ok(self)
    }
}

impl<T, E> RequestData for Result<Request<T>, E>
where
    T: RequestBody,
    E: Into<http::Error>,
{
    type Body = T;

    fn into_request(self) -> http::Result<Request<Self::Body>> {
        self.map_err(Into::into)
    }
}

/// A trait representing a request body used by the test runner.
#[allow(missing_docs)]
pub trait RequestBody {
    fn content_type(&self) -> Option<Mime> {
        None
    }

    fn into_req_body(self) -> ReqBody
    where
        Self: Sized;
}

impl RequestBody for () {
    fn into_req_body(self) -> ReqBody {
        ReqBody::new(Default::default())
    }
}

impl<'a> RequestBody for &'a str {
    fn content_type(&self) -> Option<Mime> {
        Some(mime::TEXT_PLAIN_UTF_8)
    }

    fn into_req_body(self) -> ReqBody {
        ReqBody::new(self.to_owned().into())
    }
}

impl RequestBody for String {
    fn content_type(&self) -> Option<Mime> {
        Some(mime::TEXT_PLAIN_UTF_8)
    }

    fn into_req_body(self) -> ReqBody {
        ReqBody::new(self.into())
    }
}

impl RequestBody for Body {
    fn into_req_body(self) -> ReqBody {
        ReqBody::new(self)
    }
}

/// A struct representing a response body returned from the test runner.
#[derive(Debug)]
pub struct ResBody {
    data: Vec<Bytes>,
    trailers: Option<HeaderMap>,
    content_length: Option<u64>,
}

impl ResBody {
    #[allow(missing_docs)]
    pub fn data(&self) -> &Vec<Bytes> {
        &self.data
    }

    #[allow(missing_docs)]
    pub fn trailers(&self) -> Option<&HeaderMap> {
        self.trailers.as_ref()
    }

    #[allow(missing_docs)]
    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    #[allow(missing_docs)]
    pub fn is_chunked(&self) -> bool {
        self.content_length.is_none()
    }

    #[allow(missing_docs)]
    pub fn to_bytes(&self) -> Cow<'_, [u8]> {
        match self.data.len() {
            0 => Cow::Borrowed(&[]),
            1 => Cow::Borrowed(self.data[0].as_ref()),
            _ => Cow::Owned(self.data.iter().fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&chunk);
                acc
            })),
        }
    }

    #[allow(missing_docs)]
    pub fn to_utf8(&self) -> Result<Cow<'_, str>, str::Utf8Error> {
        match self.to_bytes() {
            Cow::Borrowed(bytes) => str::from_utf8(bytes).map(Cow::Borrowed),
            Cow::Owned(bytes) => String::from_utf8(bytes)
                .map(Cow::Owned)
                .map_err(|e| e.utf8_error()),
        }
    }

    #[allow(missing_docs)]
    pub fn to_utf8_lossy(&self) -> Cow<'_, str> {
        match self.to_bytes() {
            Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
            Cow::Owned(bytes) => match String::from_utf8_lossy(&bytes) {
                Cow::Borrowed(..) => Cow::Owned(unsafe { String::from_utf8_unchecked(bytes) }),
                Cow::Owned(bytes) => Cow::Owned(bytes),
            },
        }
    }
}
