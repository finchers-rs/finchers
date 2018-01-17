#![allow(missing_docs)]

use std::fmt;
use std::error::Error;
use std::mem;
use std::marker::PhantomData;
use futures::{stream, Async, Future, Poll, Stream};
use endpoint::{Endpoint, EndpointContext, EndpointResult};
use http::{self, FromBody, Request};
use http::header::ContentLength;

pub fn body<T: FromBody>() -> Body<T> {
    Body {
        _marker: PhantomData,
    }
}

pub struct Body<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Body<T> {}

impl<T> Clone for Body<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Body<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl<T: FromBody> Endpoint for Body<T> {
    type Item = T;
    type Error = BodyError<T>;
    type Result = BodyResult<T>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        match T::is_match(ctx.request()) {
            true => Some(BodyResult {
                _marker: PhantomData,
            }),
            false => None,
        }
    }
}

#[derive(Debug)]
pub struct BodyResult<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromBody> EndpointResult for BodyResult<T> {
    type Item = T;
    type Error = BodyError<T>;
    type Future = BodyFuture<T>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        let body = request.body().expect("cannot take the request body twice");
        if T::validate(request) {
            let len = request
                .header::<ContentLength>()
                .map_or(0, |&ContentLength(len)| len as usize);
            BodyFuture::Receiving(body, Vec::with_capacity(len))
        } else {
            BodyFuture::InvalidRequest(body.for_each(|_| Ok(())))
        }
    }
}

#[allow(missing_debug_implementations)]
pub enum BodyFuture<T> {
    InvalidRequest(stream::ForEach<http::Body, fn(http::Chunk) -> Result<(), http::Error>, Result<(), http::Error>>),
    Receiving(http::Body, Vec<u8>),
    Done(PhantomData<fn() -> T>),
}

impl<T: FromBody> Future for BodyFuture<T> {
    type Item = T;
    type Error = Result<BodyError<T>, http::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::BodyFuture::*;
        match mem::replace(self, BodyFuture::Done(PhantomData)) {
            InvalidRequest(mut f) => match f.poll().map_err(Err)? {
                Async::Ready(()) => Err(Ok(BodyError::InvalidRequest)),
                Async::NotReady => {
                    *self = BodyFuture::InvalidRequest(f);
                    Ok(Async::NotReady)
                }
            },
            Receiving(mut body, mut buf) => loop {
                match body.poll().map_err(Err)? {
                    Async::Ready(Some(item)) => {
                        buf.extend_from_slice(&item);
                        continue;
                    }
                    Async::Ready(None) => {
                        let body = T::from_body(buf).map_err(|e| Ok(BodyError::FromBody(e)))?;
                        break Ok(Async::Ready(body));
                    }
                    Async::NotReady => {
                        *self = Receiving(body, buf);
                        break Ok(Async::NotReady);
                    }
                }
            },
            Done(..) => panic!("cannot resolve twice"),
        }
    }
}

#[allow(missing_docs)]
pub enum BodyError<T: FromBody = ()> {
    InvalidRequest,
    FromBody(T::Error),
}

impl<T: FromBody> fmt::Debug for BodyError<T>
where
    T::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BodyError::InvalidRequest => f.debug_struct("BadRequest").finish(),
            BodyError::FromBody(ref e) => e.fmt(f),
        }
    }
}

impl<T: FromBody> fmt::Display for BodyError<T>
where
    T::Error: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BodyError::InvalidRequest => f.write_str("invalid request"),
            BodyError::FromBody(ref e) => e.fmt(f),
        }
    }
}

impl<T: FromBody> Error for BodyError<T>
where
    T::Error: Error,
{
    fn description(&self) -> &str {
        match *self {
            BodyError::InvalidRequest => "invalid request",
            BodyError::FromBody(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            BodyError::InvalidRequest => None,
            BodyError::FromBody(ref e) => Some(e),
        }
    }
}

impl<T: FromBody> PartialEq for BodyError<T>
where
    T::Error: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (&BodyError::InvalidRequest, &BodyError::InvalidRequest) => true,
            (&BodyError::FromBody(ref l), &BodyError::FromBody(ref r)) => l.eq(r),
            _ => false,
        }
    }
}
