use bytes::Bytes;
use error::HttpError;
use futures::Async::*;
use futures::{Poll, Stream};
use http::{header, Response, StatusCode};
use input::Input;
use std::{fmt, io};

pub struct Body {
    inner: Inner,
}

enum Inner {
    Empty,
    Once(Option<Bytes>),
    Stream(Box<Stream<Item = Bytes, Error = io::Error> + Send>),
}

impl Body {
    pub fn empty() -> Body {
        Body { inner: Inner::Empty }
    }

    pub fn once<T>(body: T) -> Body
    where
        T: Into<Bytes>,
    {
        Body {
            inner: Inner::Once(Some(body.into())),
        }
    }

    pub fn wrap_stream<T>(stream: T) -> Body
    where
        T: Stream<Item = Bytes, Error = io::Error> + Send + 'static,
    {
        Body {
            inner: Inner::Stream(Box::new(stream)),
        }
    }
}

impl Stream for Body {
    type Item = Bytes;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.inner {
            Inner::Empty => Ok(Ready(None)),
            Inner::Once(ref mut chunk) => Ok(Ready(chunk.take())),
            Inner::Stream(ref mut stream) => stream.poll(),
        }
    }
}

pub type Output = Response<Body>;

pub trait Responder {
    type Error: HttpError;

    fn respond(self, input: &Input) -> Result<Output, Self::Error>;
}

impl<T> Responder for Response<T>
where
    T: Into<Body>,
{
    type Error = !;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        Ok(self.map(Into::into))
    }
}

/// A trait for constructing an HTTP response from the value.
pub trait HttpStatus {
    /// Returns a HTTP status code associated with this type
    fn status_code(&self) -> StatusCode;
}

pub struct Debug<T> {
    value: T,
    pretty: bool,
}

impl<T: fmt::Debug> Debug<T> {
    pub fn new(value: T) -> Debug<T> {
        Debug { value, pretty: false }
    }

    pub fn pretty(&mut self, enabled: bool) {
        self.pretty = enabled;
    }
}

impl<T: fmt::Debug> Responder for Debug<T> {
    type Error = !;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let body = if self.pretty {
            format!("{:#?}", self.value)
        } else {
            format!("{:?}", self.value)
        };
        Ok(make_text_response(body))
    }
}

pub struct Display<T> {
    value: T,
    status: StatusCode,
}

impl<T: fmt::Display> Display<T> {
    pub fn new(value: T) -> Display<T> {
        Display::with_status(value, StatusCode::OK)
    }

    pub fn with_status(value: T, status: StatusCode) -> Display<T> {
        Display { value, status }
    }
}

impl<T: fmt::Display> Responder for Display<T> {
    type Error = !;

    fn respond(self, _: &Input) -> Result<Output, Self::Error> {
        let mut response = make_text_response(self.value.to_string());
        *response.status_mut() = self.status;
        Ok(response)
    }
}

fn make_text_response(body: String) -> Response<Body> {
    let body_len = body.len().to_string();
    let mut response = Response::new(Body::once(body));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
    response
        .headers_mut()
        .insert(header::CONTENT_LENGTH, body_len.parse().unwrap());
    response
}
