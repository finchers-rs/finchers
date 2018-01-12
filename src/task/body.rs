use std::marker::PhantomData;
use std::mem;
use futures::{Async, Future, Poll, Stream};
use futures::future::{self, FutureResult};
use http::{self, FromBody, FromBodyError, HttpError};
use http::header::ContentLength;
use task::{Task, TaskContext};

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Body<T> {
    pub(crate) _marker: PhantomData<fn() -> T>,
}

impl<T: FromBody> Task for Body<T> {
    type Item = T;
    type Error = FromBodyError<T::Error>;
    type Future = BodyFuture<T>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        if !T::validate(ctx.request()) {
            return BodyFuture::BadRequest;
        }

        let body = ctx.take_body().expect("cannot take the request body twice");
        let len = ctx.request()
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
    type Error = Result<FromBodyError<T::Error>, HttpError>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::BodyFuture::*;
        match mem::replace(self, BodyFuture::Done(PhantomData)) {
            BadRequest => Err(Ok(FromBodyError::BadRequest)),
            Receiving(mut body, mut buf) => loop {
                match body.poll().map_err(Err)? {
                    Async::Ready(Some(item)) => {
                        buf.extend_from_slice(&item);
                        continue;
                    }
                    Async::Ready(None) => {
                        let body = T::from_body(buf).map_err(|e| Ok(FromBodyError::FromBody(e)))?;
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
#[derive(Debug)]
pub struct BodyStream<E> {
    pub(crate) _marker: PhantomData<fn() -> E>,
}

impl<E> Task for BodyStream<E> {
    type Item = http::Body;
    type Error = E;
    type Future = FutureResult<Self::Item, Result<Self::Error, HttpError>>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let body = ctx.take_body().expect("cannot take a body twice");
        future::ok(body)
    }
}
