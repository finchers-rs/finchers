//! The basic facilities for testing endpoints.

use std::io;

use bytes::Buf;
use futures::{future, stream, Async, Future, Stream};
use http::header;
use http::header::HeaderMap;
use http::Response;
use hyper::body::Payload;
use tokio::runtime::current_thread::Runtime;

use endpoint::Endpoint;
use error;
use output::Output;

use super::app::app_service::{AppFuture, AppService};
use super::blocking::{with_set_runtime_mode, RuntimeMode};

pub use self::request::TestRequest;
pub use self::response::TestResponse;

struct AnnotatedRuntime<'a>(&'a mut Runtime);

impl<'a> AnnotatedRuntime<'a> {
    fn block_on<F: Future>(&mut self, mut future: F) -> Result<F::Item, F::Error> {
        self.0.block_on(future::poll_fn(move || {
            with_set_runtime_mode(RuntimeMode::CurrentThread, || future.poll())
        }))
    }
}

/// A helper function for creating a new `TestRunner` from the specified endpoint.
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
        Req: TestRequest,
        F: FnOnce(AppFuture<'a, E>, &mut AnnotatedRuntime<'_>) -> R,
    {
        let mut request = request
            .into_request()
            .expect("failed to construct a request");
        if let Some(headers) = self.headers.clone() {
            request.headers_mut().extend(headers);
        }
        if let Some(len) = request.body().content_length() {
            request
                .headers_mut()
                .entry(header::CONTENT_LENGTH)
                .unwrap()
                .or_insert(
                    len.to_string()
                        .parse()
                        .expect("should be a valid header value"),
                );
        }

        let future = AppService::new(&self.endpoint).dispatch(request);

        f(future, &mut AnnotatedRuntime(&mut self.rt))
    }

    /// Applys the given request to the inner endpoint and retrieves the result of returned future.
    pub fn apply_raw<'a>(&'a mut self, request: impl TestRequest) -> error::Result<E::Output>
    where
        E: Endpoint<'a>,
    {
        self.apply_inner(request, |mut future, rt| {
            rt.block_on(future::poll_fn(|| future.poll_endpoint()))
        })
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn apply<'a, T>(&'a mut self, request: impl TestRequest) -> error::Result<T>
    where
        E: Endpoint<'a, Output = (T,)>,
    {
        self.apply_raw(request).map(|(x,)| x)
    }

    /// Retrieves the retrieves the result of future returned from `Endpoint::apply`,
    /// and converting it into an HTTP response by calling `Output::respond`.
    pub fn apply_output<'a>(
        &'a mut self,
        request: impl TestRequest,
    ) -> error::Result<Response<<E::Output as Output>::Body>>
    where
        E: Endpoint<'a>,
        E::Output: Output,
    {
        self.apply_inner(request, |mut future, rt| {
            rt.block_on(future::poll_fn(|| future.poll_output()))
        })
    }

    /// Gets the response of specified HTTP request.
    pub fn apply_all<'a>(&'a mut self, request: impl TestRequest) -> TestResponse
    where
        E: Endpoint<'a>,
        E::Output: Output,
    {
        self.apply_inner(request, |mut future, rt| {
            let response = rt.block_on(future::poll_fn(|| future.poll_all())).unwrap();
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

            TestResponse {
                parts,
                data,
                trailers,
                content_length,
            }
        })
    }

    /// Returns a reference to the underlying Tokio runtime.
    pub fn runtime(&mut self) -> &mut Runtime {
        &mut self.rt
    }
}

mod request {
    use http;
    use http::header;
    use http::{Request, Uri};
    use hyper::body::Body;
    use mime;
    use mime::Mime;

    use input::ReqBody;

    /// A trait representing a request used by the test runner.
    ///
    /// The implementors of this trait is currently as follows:
    ///
    /// * `&str` and `String`. It will be converted to a GET request with the specified URI.
    /// * `http::Request<T>`, where the type of message body `T` is one of the following:
    ///   - `()`
    ///   - `&str` or `String` (they also insert the value of `content-type` and `content-length` if missing)
    ///   - `hyper::Body` (it also inserts the value of `content-length` if mentioned)
    /// * `http::request::Builder` and `&mut http::request::Builder`, with an empty body.
    /// * `Result<T: TestRequest, E: Into<Error>>`
    pub trait TestRequest: TestRequestImpl {}
    impl<T: TestRequestImpl> TestRequest for T {}

    pub trait TestRequestImpl {
        fn into_request(self) -> http::Result<Request<ReqBody>>;
    }

    impl<'a> TestRequestImpl for &'a str {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            (*self).parse::<Uri>()?.into_request()
        }
    }

    impl TestRequestImpl for String {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            self.parse::<Uri>()?.into_request()
        }
    }

    impl TestRequestImpl for Uri {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            (&self).into_request()
        }
    }

    impl<'a> TestRequestImpl for &'a Uri {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            let mut request =
                Request::get(self.path_and_query().map(|s| s.as_str()).unwrap_or("/"));
            if let Some(authority) = self.authority_part() {
                match authority.port() {
                    Some(port) => {
                        request.header(header::HOST, format!("{}:{}", authority.host(), port));
                    }
                    None => {
                        request.header(header::HOST, authority.host());
                    }
                }
            }
            request.body(ReqBody::new(Default::default()))
        }
    }

    impl<T: RequestBody> TestRequestImpl for Request<T> {
        fn into_request(mut self) -> http::Result<Request<ReqBody>> {
            if let Some(mime) = self.body().content_type() {
                self.headers_mut()
                    .entry(header::CONTENT_TYPE)
                    .unwrap()
                    .or_insert(
                        mime.as_ref()
                            .parse()
                            .expect("should be a valid header value"),
                    );
            }
            Ok(self.map(|bd| bd.into_req_body()))
        }
    }

    impl TestRequestImpl for http::request::Builder {
        fn into_request(mut self) -> http::Result<Request<ReqBody>> {
            self.body(ReqBody::new(Default::default()))
        }
    }

    impl<'a> TestRequestImpl for &'a mut http::request::Builder {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            self.body(ReqBody::new(Default::default()))
        }
    }

    impl<T, E> TestRequestImpl for Result<T, E>
    where
        T: TestRequestImpl,
        E: Into<http::Error>,
    {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            self.map_err(Into::into)?.into_request()
        }
    }

    pub trait RequestBody: Sized {
        fn content_type(&self) -> Option<Mime> {
            None
        }
        fn into_req_body(self) -> ReqBody;
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
}

mod response {
    use std::borrow::Cow;
    use std::ops::Deref;
    use std::str;

    use bytes::Bytes;
    use http::header::HeaderMap;
    use http::response::Parts;

    /// A struct representing a response body returned from the test runner.
    #[derive(Debug)]
    pub struct TestResponse {
        pub(super) parts: Parts,
        pub(super) data: Vec<Bytes>,
        pub(super) trailers: Option<HeaderMap>,
        pub(super) content_length: Option<u64>,
    }

    impl Deref for TestResponse {
        type Target = Parts;

        fn deref(&self) -> &Self::Target {
            self.parts()
        }
    }

    impl TestResponse {
        #[allow(missing_docs)]
        pub fn parts(&self) -> &Parts {
            &self.parts
        }

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
}

#[cfg(test)]
mod tests {
    use super::{runner, TestRequest, TestResponse};
    use endpoint;
    use http::{Request, Uri};

    #[test]
    fn test_test_request() {
        fn assert_impl<T: TestRequest>(t: T) {
            drop(t)
        }

        assert_impl("/"); // &str
        assert_impl(format!("/foo/bar")); // String
        assert_impl(Uri::from_static("http://example.com/"));
        assert_impl(&Uri::from_static("/foo/bar?count=1"));
        assert_impl(Request::get("/")); // Builder
        assert_impl(Request::post("/").header("content-type", "application/json")); // &mut Builder
        assert_impl(Request::put("/").body("text")); // Result<Response<_>, Error>
    }

    #[test]
    fn test_apply_output() {
        let mut runner = runner({ endpoint::cloned("Hello") });
        let res = runner.apply_output("/");
        assert!(res.is_ok());
        let output = res.unwrap();

        assert_eq!(output.status().as_u16(), 200);
        assert!(output.headers().contains_key("content-type"));
        assert!(!output.headers().contains_key("content-length"));
        assert!(!output.headers().contains_key("server"));
    }

    #[test]
    fn test_apply_all() {
        let mut runner = runner({ endpoint::cloned("Hello") });
        let response: TestResponse = runner.apply_all("/");

        assert_eq!(response.status.as_u16(), 200);
        assert!(response.headers.contains_key("content-type"));
        assert!(response.headers.contains_key("content-length"));
        assert!(response.headers.contains_key("server"));
        assert_eq!(response.to_utf8_lossy(), "Hello");
        assert!(response.trailers().is_none());
    }

    #[test]
    fn test_default_headers() {
        let mut runner = runner({
            endpoint::apply_fn(|cx| {
                assert!(cx.headers().contains_key("origin"));
                Ok(Ok(()))
            })
        });
        runner
            .default_headers()
            .entry("origin")
            .unwrap()
            .or_insert("www.example.com".parse().unwrap());

        assert!(runner.apply_raw("/").is_ok());
    }
}
