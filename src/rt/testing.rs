#![allow(missing_docs)]

use std::borrow::Cow;

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

#[derive(Debug)]
pub struct TestRunner<E> {
    endpoint: E,
    rt: Runtime,
}

impl<E> TestRunner<E> {
    pub fn new(endpoint: E) -> TestRunner<E>
    where
        for<'e> E: Endpoint<'e>,
    {
        TestRunner {
            endpoint,
            rt: Runtime::new().expect("failed to start the runtime"),
        }
    }

    pub fn runtime(&mut self) -> &mut Runtime {
        &mut self.rt
    }

    pub fn apply_endpoint<'a>(&'a mut self, request: Request<ReqBody>) -> Result<E::Output, Error>
    where
        E: Endpoint<'a>,
    {
        let mut future = AppService::new(&self.endpoint).dispatch(request);
        self.rt.block_on(future::poll_fn(|| future.poll_endpoint()))
    }

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
}

#[derive(Debug)]
pub struct ResBody {
    data: Vec<Bytes>,
    trailers: Option<HeaderMap>,
    content_length: Option<u64>,
}

#[allow(missing_docs)]
impl ResBody {
    pub fn into_chunks(self) -> Vec<Bytes> {
        self.data
    }

    pub fn is_chunked(&self) -> bool {
        self.content_length.is_none()
    }

    pub fn trailers(&self) -> Option<&HeaderMap> {
        self.trailers.as_ref()
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
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

    pub fn to_utf8(&self) -> Cow<'_, str> {
        match self.to_bytes() {
            Cow::Borrowed(bytes) => String::from_utf8_lossy(bytes),
            Cow::Owned(bytes) => match String::from_utf8_lossy(&bytes) {
                Cow::Borrowed(..) => Cow::Owned(unsafe { String::from_utf8_unchecked(bytes) }),
                Cow::Owned(bytes) => Cow::Owned(bytes),
            },
        }
    }
}
