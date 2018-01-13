#![allow(missing_docs)]

use std::fmt;
use std::error::Error;
use std::mem;
use std::marker::PhantomData;
use futures::{Async, Future, Poll, Stream};
use endpoint::{Endpoint, EndpointContext, EndpointResult};
use http::{self, FromBody, Request};
use http::header::ContentLength;
use errors::ErrorResponder;

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
        if !T::validate(request) {
            return BodyFuture::BadRequest;
        }

        let body = request.body().expect("cannot take the request body twice");
        let len = request
            .header::<ContentLength>()
            .map_or(0, |&ContentLength(len)| len as usize);
        BodyFuture::Receiving(body, Vec::with_capacity(len))
    }
}

#[derive(Debug)]
pub enum BodyFuture<T> {
    BadRequest,
    Receiving(http::Body, Vec<u8>),
    Done(PhantomData<fn() -> T>),
}

impl<T: FromBody> Future for BodyFuture<T> {
    type Item = T;
    type Error = Result<BodyError<T>, http::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::BodyFuture::*;
        match mem::replace(self, BodyFuture::Done(PhantomData)) {
            BadRequest => Err(Ok(BodyError::BadRequest)),
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
    BadRequest,
    FromBody(T::Error),
}

impl<T: FromBody> fmt::Debug for BodyError<T>
where
    T::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BodyError::BadRequest => f.debug_struct("BadRequest").finish(),
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
            BodyError::BadRequest => f.write_str("bad request"),
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
            BodyError::BadRequest => "bad request",
            BodyError::FromBody(ref e) => e.description(),
        }
    }
}

impl<T: FromBody> ErrorResponder for BodyError<T>
where
    T::Error: Error,
{
}

impl<T: FromBody> PartialEq for BodyError<T>
where
    T::Error: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (&BodyError::BadRequest, &BodyError::BadRequest) => true,
            (&BodyError::FromBody(ref l), &BodyError::FromBody(ref r)) => l.eq(r),
            _ => false,
        }
    }
}
