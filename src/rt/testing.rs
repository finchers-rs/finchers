//! The basic facilities for testing endpoints.

use std::borrow::Cow;
use std::io;
use std::str;

use bytes::{Buf, Bytes};
use futures::{future, stream, Async, Future, Stream};
use http::header;
use http::header::HeaderMap;
use http::{Request, Response};
use hyper::body::{Body, Payload};
use tokio::runtime::current_thread::Runtime;

use endpoint::Endpoint;
use error::Error;
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

    fn apply_inner<'a, Bd, F, R>(&'a mut self, request: Request<Bd>, f: F) -> R
    where
        E: Endpoint<'a>,
        Bd: ReqBody,
        F: FnOnce(AppFuture<'a, E>, &mut AnnotatedRuntime<'_>) -> R,
    {
        let (mut parts, body) = request.into_parts();
        if let Some(mime) = body.content_type() {
            parts.headers.insert(
                header::CONTENT_TYPE,
                mime.as_ref()
                    .parse()
                    .expect("should be a valid header value"),
            );
        }
        if let Some(headers) = self.headers.clone() {
            parts.headers.extend(headers);
        }

        let body = body.into_req_body();
        if let Some(len) = body.content_length() {
            parts.headers.insert(
                header::CONTENT_LENGTH,
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
    pub fn apply_endpoint<'a, Bd>(&'a mut self, request: Request<Bd>) -> Result<E::Output, Error>
    where
        E: Endpoint<'a>,
        Bd: ReqBody,
    {
        self.apply_inner(request, |mut future, rt| {
            rt.block_on(future::poll_fn(|| future.poll_endpoint()))
        })
    }

    /// Retrieves the retrieves the result of future returned from `Endpoint::apply`,
    /// and converting it into an HTTP response by calling `Output::respond`.
    pub fn apply_output<'a, Bd>(
        &'a mut self,
        request: Request<Bd>,
    ) -> Result<Response<<E::Output as Output>::Body>, Error>
    where
        E: Endpoint<'a>,
        E::Output: Output,
        Bd: ReqBody,
    {
        self.apply_inner(request, |mut future, rt| {
            rt.block_on(future::poll_fn(|| future.poll_output()))
        })
    }

    /// Gets the response of specified HTTP request.
    pub fn apply_all<'a, Bd>(&'a mut self, request: Request<Bd>) -> Response<ResBody>
    where
        E: Endpoint<'a>,
        E::Output: Output,
        Bd: ReqBody,
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

/// A trait representing a request body used by the test runner.
pub trait ReqBody: req_body::ReqBodyImpl {}

impl ReqBody for () {}
impl<'a> ReqBody for &'a str {}
impl ReqBody for String {}
impl ReqBody for Body {}

mod req_body {
    use hyper::body::Body;
    use input::ReqBody;
    use mime;
    use mime::Mime;

    #[doc(hidden)]
    pub trait ReqBodyImpl {
        fn content_type(&self) -> Option<Mime> {
            None
        }

        fn into_req_body(self) -> ReqBody
        where
            Self: Sized;
    }

    impl ReqBodyImpl for () {
        fn into_req_body(self) -> ReqBody
        where
            Self: Sized,
        {
            ReqBody::new(Default::default())
        }
    }

    impl<'a> ReqBodyImpl for &'a str {
        fn content_type(&self) -> Option<Mime> {
            Some(mime::TEXT_PLAIN_UTF_8)
        }

        fn into_req_body(self) -> ReqBody
        where
            Self: Sized,
        {
            ReqBody::new(self.to_owned().into())
        }
    }

    impl ReqBodyImpl for String {
        fn content_type(&self) -> Option<Mime> {
            Some(mime::TEXT_PLAIN_UTF_8)
        }

        fn into_req_body(self) -> ReqBody
        where
            Self: Sized,
        {
            ReqBody::new(self.into())
        }
    }

    impl ReqBodyImpl for Body {
        fn into_req_body(self) -> ReqBody
        where
            Self: Sized,
        {
            ReqBody::new(self)
        }
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
