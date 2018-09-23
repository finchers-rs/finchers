#![allow(missing_docs)]

use futures::Poll;
use http::header::HeaderMap;
use hyper::body::{Body, Chunk, Payload as _Payload};

use error::{fail, Error};

#[derive(Debug)]
pub struct Payload {
    body: Body,
}

impl Payload {
    pub fn poll_data(&mut self) -> Poll<Option<Chunk>, Error> {
        self.body.poll_data().map_err(fail)
    }

    pub fn poll_trailers(&mut self) -> Poll<Option<HeaderMap>, Error> {
        self.body.poll_trailers().map_err(fail)
    }

    pub fn is_end_stream(&self) -> bool {
        self.body.is_end_stream()
    }

    pub fn content_length(&self) -> Option<u64> {
        self.body.content_length()
    }
}

/// An asyncrhonous stream to receive the chunks of incoming request body.
#[derive(Debug)]
pub struct ReqBody(Option<Body>);

impl ReqBody {
    /// Create an instance of `RequestBody` from `hyper::Body`.
    pub fn from_hyp(body: Body) -> ReqBody {
        ReqBody(Some(body))
    }

    #[allow(missing_docs)]
    pub fn payload(&mut self) -> Option<Payload> {
        self.0.take().map(|body| Payload { body })
    }

    pub fn is_gone(&self) -> bool {
        self.0.is_none()
    }
}
