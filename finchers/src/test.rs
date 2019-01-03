//! The basic facilities for testing endpoints.
//!
//! # Example
//!
//! ```ignore
//! # #[macro_use]
//! # extern crate finchers;
//! # use finchers::test;
//! # use finchers::prelude::*;
//! # fn main() {
//! let endpoint = path!(@get / "greeting" / String)
//!     .map(|name: String| format!("Hello, {}.", name));
//!
//! // Create an instance of TestRunner from an endpoint.
//! let mut runner = test::runner(endpoint);
//!
//! let response = runner
//!     .perform("http://www.example.com/greeting/Alice")
//!     .unwrap();
//! assert_eq!(response.status().as_u16(), 200);
//! assert!(response.headers().contains_key("content-type"));
//! assert_eq!(response.body().to_utf8().unwrap(), "Hello, Alice.");
//! # }
//! ```
//!
//! Validates the result of the endpoint without converting to HTTP response.
//!
//! ```
//! # use finchers::test;
//! # use finchers::prelude::*;
//! use finchers::error::Result;
//!
//! // A user-defined type which does not implement `Output`.
//! struct Credential;
//! let endpoint = endpoint::unit().map(|| Credential);
//!
//! let mut runner = test::runner(endpoint);
//!
//! let result: Result<Credential> = runner.apply("/");
//!
//! assert!(result.is_ok());
//! ```

use {
    crate::{
        endpoint::Endpoint,
        error,
        service::{AppFuture, AppService},
    },
    bytes::Bytes,
    futures::{future, Poll},
    http::{
        header::{self, HeaderMap, HeaderName, HeaderValue},
        Request, Uri,
    },
    hyper::body::Payload,
    mime::Mime,
    std::io,
    tokio::runtime::current_thread::Runtime,
};

// ====

fn or_insert(headers: &mut HeaderMap, name: HeaderName, value: &'static str) {
    headers
        .entry(name)
        .unwrap()
        .or_insert_with(|| HeaderValue::from_static(value));
}

/// A trait representing the conversion into an HTTP request.
///
/// This trait is internally used by the test runner.
pub trait TestRequest: self::imp::TestRequestImpl {}

/// A trait representing the conversion into a message body in HTTP requests.
///
/// This trait is internally used by the test runner.
pub trait IntoReqBody: self::imp::IntoReqBodyImpl {}

// ==== ReqBody ====

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ReqBody(Option<Bytes>);

impl Payload for ReqBody {
    type Data = io::Cursor<Bytes>;
    type Error = io::Error;

    #[inline]
    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        Ok(self.0.take().map(io::Cursor::new).into())
    }

    #[inline]
    fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Self::Error> {
        Ok(None.into())
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.0.is_none()
    }

    #[inline]
    fn content_length(&self) -> Option<u64> {
        self.0.as_ref().map(|x| x.len() as u64)
    }
}

/// A helper function for creating a new `TestRunner` from the specified endpoint.
pub fn runner<E>(endpoint: E) -> TestRunner<E>
where
    E: Endpoint<ReqBody>,
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
    rt: Runtime,
    default_headers: Option<HeaderMap>,
}

impl<E> TestRunner<E>
where
    E: Endpoint<ReqBody>,
{
    /// Create a `TestRunner` from the specified endpoint.
    pub fn new(endpoint: E) -> io::Result<TestRunner<E>> {
        Runtime::new().map(|rt| TestRunner::with_runtime(endpoint, rt))
    }

    /// Create a `TestRunner` from the specified endpoint with a Tokio runtime.
    pub fn with_runtime(endpoint: E, rt: Runtime) -> TestRunner<E> {
        TestRunner {
            endpoint,
            rt,
            default_headers: None,
        }
    }

    /// Returns a reference to the header map, whose values are set before
    /// applying the request to endpoint.
    pub fn default_headers(&mut self) -> &mut HeaderMap {
        self.default_headers.get_or_insert_with(Default::default)
    }

    /// Returns a reference to the instance of `Endpoint` owned by this runner.
    pub fn endpoint(&mut self) -> &mut E {
        &mut self.endpoint
    }

    /// Returns a reference to the Tokio runtime managed by this runner.
    pub fn runtime(&mut self) -> &mut Runtime {
        &mut self.rt
    }

    fn prepare_request(&self, request: impl TestRequest) -> http::Result<Request<ReqBody>> {
        let mut request = request.into_request()?;

        if let Some(ref default_headers) = self.default_headers {
            for (k, v) in default_headers {
                request.headers_mut().append(k, v.clone());
            }
        }

        if let Some(len) = request.body().content_length() {
            request
                .headers_mut()
                .entry(header::CONTENT_LENGTH)
                .unwrap()
                .or_insert_with(|| {
                    len.to_string()
                        .parse()
                        .expect("should be a valid header value")
                });
        }

        or_insert(request.headers_mut(), header::HOST, "localhost");
        or_insert(
            request.headers_mut(),
            header::USER_AGENT,
            concat!("finchers/", env!("CARGO_PKG_VERSION")),
        );

        Ok(request)
    }

    fn apply_inner<'a, F, R>(&'a mut self, request: impl TestRequest, f: F) -> R
    where
        F: FnOnce(AppFuture<ReqBody, &E>, &mut Runtime) -> R,
    {
        let request = self
            .prepare_request(request)
            .expect("failed to construct a request");

        let future = AppService::new(&self.endpoint).dispatch(request);

        f(future, &mut self.rt)
    }

    /// Applies the given request to the inner endpoint and retrieves the result of returned future.
    ///
    /// This method is available only if the output of endpoint is a tuple with a single element.
    /// If the output type is an unit or the tuple contains more than one element, use `apply_raw` instead.
    #[inline]
    pub fn apply<T>(&mut self, request: impl TestRequest) -> error::Result<T>
    where
        E: Endpoint<ReqBody, Output = (T,)>,
    {
        self.apply_raw(request).map(|(x,)| x)
    }

    /// Applies the given request to the inner endpoint and retrieves the result of returned future
    /// *without peeling tuples*.
    pub fn apply_raw(&mut self, request: impl TestRequest) -> error::Result<E::Output> {
        self.apply_inner(request, |mut future, rt| {
            rt.block_on(future::poll_fn(|| future.poll_apply()))
        })
    }
}

mod imp {
    use super::*;

    pub trait TestRequestImpl {
        fn into_request(self) -> http::Result<Request<ReqBody>>;
    }

    impl<'a> TestRequest for &'a str {}
    impl<'a> TestRequestImpl for &'a str {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            (*self).parse::<Uri>()?.into_request()
        }
    }

    impl TestRequest for String {}
    impl TestRequestImpl for String {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            self.parse::<Uri>()?.into_request()
        }
    }

    impl TestRequest for Uri {}
    impl TestRequestImpl for Uri {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            (&self).into_request()
        }
    }

    impl<'a> TestRequest for &'a Uri {}
    impl<'a> TestRequestImpl for &'a Uri {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            let path = self.path_and_query().map(|s| s.as_str()).unwrap_or("/");
            let mut request = Request::get(path) //
                .body(ReqBody(Some(Bytes::new())))?;

            if let Some(authority) = self.authority_part() {
                request
                    .headers_mut()
                    .entry(header::HOST)
                    .unwrap()
                    .or_insert(match authority.port_part() {
                        Some(port) => format!("{}:{}", authority.host(), port).parse()?,
                        None => authority.host().parse()?,
                    });
            }

            Ok(request)
        }
    }

    impl<T: IntoReqBody> TestRequest for Request<T> {}
    impl<T: IntoReqBody> TestRequestImpl for Request<T> {
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

    impl TestRequest for http::request::Builder {}
    impl TestRequestImpl for http::request::Builder {
        fn into_request(mut self) -> http::Result<Request<ReqBody>> {
            self.body(ReqBody(Some(Bytes::new())))
        }
    }

    impl<'a> TestRequest for &'a mut http::request::Builder {}
    impl<'a> TestRequestImpl for &'a mut http::request::Builder {
        fn into_request(self) -> http::Result<Request<ReqBody>> {
            self.body(ReqBody(Some(Bytes::new())))
        }
    }

    impl<T, E> TestRequest for Result<T, E>
    where
        T: TestRequest,
        E: Into<http::Error>,
    {
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

    // ==== IntoReqBody ====

    pub trait IntoReqBodyImpl: Sized {
        fn content_type(&self) -> Option<Mime> {
            None
        }
        fn into_req_body(self) -> ReqBody;
    }

    impl IntoReqBody for () {}
    impl IntoReqBodyImpl for () {
        fn into_req_body(self) -> ReqBody {
            ReqBody(Some(Bytes::new()))
        }
    }

    impl<'a> IntoReqBody for &'a [u8] {}
    impl<'a> IntoReqBodyImpl for &'a [u8] {
        fn into_req_body(self) -> ReqBody {
            ReqBody(Some(Bytes::from(self)))
        }
    }

    impl IntoReqBody for Vec<u8> {}
    impl<'a> IntoReqBodyImpl for Vec<u8> {
        fn into_req_body(self) -> ReqBody {
            ReqBody(Some(Bytes::from(self)))
        }
    }

    impl<'a> IntoReqBody for &'a str {}
    impl<'a> IntoReqBodyImpl for &'a str {
        fn content_type(&self) -> Option<Mime> {
            Some(mime::TEXT_PLAIN_UTF_8)
        }

        fn into_req_body(self) -> ReqBody {
            ReqBody(Some(Bytes::from(self)))
        }
    }

    impl IntoReqBody for String {}
    impl IntoReqBodyImpl for String {
        fn content_type(&self) -> Option<Mime> {
            Some(mime::TEXT_PLAIN_UTF_8)
        }

        fn into_req_body(self) -> ReqBody {
            ReqBody(Some(Bytes::from(self)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::endpoint;
    use crate::endpoint::Endpoint;
    use matches::assert_matches;

    #[test]
    fn test_test_request() {
        fn assert_impl<T: TestRequest>(t: T) {
            drop(t)
        }

        assert_impl("/"); // &str
        assert_impl("/foo/bar".to_string()); // String
        assert_impl(Uri::from_static("http://example.com/"));
        assert_impl(&Uri::from_static("/foo/bar?count=1"));
        assert_impl(Request::get("/")); // Builder
        assert_impl(Request::post("/").header("content-type", "application/json")); // &mut Builder
        assert_impl(Request::put("/").body("text")); // Result<Response<_>, Error>
    }

    #[test]
    fn test_host_useragent() {
        let mut runner = runner({
            crate::endpoint::apply_fn(|_cx| {
                Ok(crate::future::poll_fn(|cx| {
                    let host = cx.headers().get(header::HOST).cloned();
                    let user_agent = cx.headers().get(header::USER_AGENT).cloned();
                    Ok::<_, crate::error::Error>((host, user_agent).into())
                }))
            })
        });

        assert_matches!(
            runner.apply_raw("/"),
            Ok((Some(ref host), Some(ref user_agent)))
                if host == "localhost" &&
                   user_agent.to_str().unwrap().starts_with("finchers/")
        );

        assert_matches!(
            runner.apply_raw("http://www.example.com/path/to"),
            Ok((Some(ref host), Some(ref user_agent)))
                if host == "www.example.com" &&
                   user_agent.to_str().unwrap().starts_with("finchers/")
        );

        assert_matches!(
            runner.apply_raw(
                Request::get("/path/to")
                    .header(header::USER_AGENT, "custom/0.0.0")),
            Ok((Some(ref host), Some(ref user_agent)))
                if host == "localhost" &&
                   user_agent.to_str().unwrap() == "custom/0.0.0"

        );
    }

    #[test]
    fn test_default_headers() {
        let mut runner = runner({
            endpoint::unit().wrap(endpoint::wrapper::before_apply(|cx| {
                assert!(cx.headers().contains_key(header::ORIGIN));
                Ok(())
            }))
        });
        runner
            .default_headers()
            .entry(header::ORIGIN)
            .unwrap()
            .or_insert("www.example.com".parse().unwrap());

        assert!(runner.apply_raw("/").is_ok());
    }
}
