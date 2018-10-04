//! The basic facilities for testing endpoints.

#![allow(missing_docs)]

use std::borrow::Cow;
use std::io;
use std::str;

use bytes::{Buf, Bytes};
use futures::{future, stream, Async, Future, Stream};
use http::header::HeaderMap;
use http::{Request, Response};
use hyper::body::Payload;
use tokio::runtime::current_thread::Runtime;

use endpoint::Endpoint;
use error::Error;
use input::ReqBody;
use output::Output;

use super::app::AppService;

/// A test runner for emulating the behavior of endpoints in the server.
///
/// It uses internally the current thread version of Tokio runtime for executing
/// asynchronous processes.
#[derive(Debug)]
pub struct TestRunner<E> {
    endpoint: E,
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
            rt: Runtime::new()?,
        })
    }

    /// Create a `TestRunner` from the specified endpoint with a Tokio runtime.
    pub fn with_runtime(endpoint: E, rt: Runtime) -> TestRunner<E> {
        TestRunner { endpoint, rt }
    }

    /// Applys the given request to the inner endpoint and retrieves the result of returned future.
    pub fn apply_endpoint<'a>(&'a mut self, request: Request<ReqBody>) -> Result<E::Output, Error>
    where
        E: Endpoint<'a>,
    {
        let mut future = AppService::new(&self.endpoint).dispatch(request);
        self.rt.block_on(future::poll_fn(|| future.poll_endpoint()))
    }

    /// Retrieves the retrieves the result of future returned from `Endpoint::apply`,
    /// and converting it into an HTTP response by calling `Output::respond`.
    pub fn apply_output<'a>(
        &'a mut self,
        request: Request<ReqBody>,
    ) -> Result<Response<<E::Output as Output>::Body>, Error>
    where
        E: Endpoint<'a>,
        E::Output: Output,
    {
        let mut future = AppService::new(&self.endpoint).dispatch(request);
        self.rt.block_on(future::poll_fn(|| future.poll_output()))
    }

    /// Gets the response of specified HTTP request.
    pub fn apply_all<'a>(&'a mut self, request: Request<ReqBody>) -> Response<ResBody>
    where
        E: Endpoint<'a>,
        E::Output: Output,
    {
        let mut future = AppService::new(&self.endpoint).dispatch(request);

        let response = self.rt.block_on(future::poll_fn(|| future.poll())).unwrap();
        let (parts, mut payload) = response.into_parts();

        // construct ResBody
        let content_length = payload.content_length();

        let data = self
            .rt
            .block_on(
                stream::poll_fn(|| match payload.poll_data() {
                    Ok(Async::Ready(data)) => Ok(Async::Ready(data.map(Buf::collect))),
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Err(err) => Err(err),
                }).collect(),
            ).expect("error during sending the response body.");

        let trailers = self
            .rt
            .block_on(future::poll_fn(|| payload.poll_trailers()))
            .expect("error during sending trailers.");

        let body = ResBody {
            data,
            trailers,
            content_length,
        };

        Response::from_parts(parts, body)
    }

    /// Returns a reference to the underlying Tokio runtime.
    pub fn runtime(&mut self) -> &mut Runtime {
        &mut self.rt
    }
}

#[derive(Debug)]
pub struct ResBody {
    data: Vec<Bytes>,
    trailers: Option<HeaderMap>,
    content_length: Option<u64>,
}

impl ResBody {
    pub fn data(&self) -> &Vec<Bytes> {
        &self.data
    }

    pub fn trailers(&self) -> Option<&HeaderMap> {
        self.trailers.as_ref()
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    pub fn is_chunked(&self) -> bool {
        self.content_length.is_none()
    }

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

    pub fn to_utf8(&self) -> Result<Cow<'_, str>, str::Utf8Error> {
        match self.to_bytes() {
            Cow::Borrowed(bytes) => str::from_utf8(bytes).map(Cow::Borrowed),
            Cow::Owned(bytes) => String::from_utf8(bytes)
                .map(Cow::Owned)
                .map_err(|e| e.utf8_error()),
        }
    }

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
