use std::marker::PhantomData;
use std::mem;
use futures::{Async, Future, Poll, Stream};
use http::{self, FromBody};
use http::header::ContentLength;
use task::{Task, TaskContext};

#[derive(Debug)]
pub struct Body<T, E> {
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Default for Body<T, E> {
    fn default() -> Self {
        Body {
            _marker: PhantomData,
        }
    }
}

impl<T, E> Task for Body<T, E>
where
    T: FromBody,
    E: From<http::HttpError> + From<T::Error>,
{
    type Item = T;
    type Error = E;
    type Future = BodyFuture<T, E>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        if let Err(e) = T::validate(ctx.request()) {
            return BodyFuture::BadRequest(e.into());
        }

        let body = ctx.take_body().expect("cannot take the request body twice");
        let len = ctx.request()
            .header::<ContentLength>()
            .map_or(0, |&ContentLength(len)| len as usize);
        BodyFuture::Receiving(body, Vec::with_capacity(len))
    }
}

#[derive(Debug)]
pub enum BodyFuture<T, E> {
    BadRequest(E),
    Receiving(http::Body, Vec<u8>),
    Done(PhantomData<fn() -> (T, E)>),
}

impl<T, E> Future for BodyFuture<T, E>
where
    T: FromBody,
    E: From<http::HttpError> + From<T::Error>,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match mem::replace(self, BodyFuture::Done(PhantomData)) {
            BodyFuture::BadRequest(err) => Err(err),
            BodyFuture::Receiving(mut body, mut buf) => loop {
                match body.poll() {
                    Ok(Async::Ready(Some(item))) => {
                        buf.extend_from_slice(&item);
                        continue;
                    }
                    Ok(Async::Ready(None)) => {
                        let body = T::from_body(buf)?;
                        break Ok(body.into());
                    }
                    Ok(Async::NotReady) => {
                        *self = BodyFuture::Receiving(body, buf);
                        break Ok(Async::NotReady);
                    }
                    Err(err) => {
                        break Err(err.into());
                    }
                }
            },
            BodyFuture::Done(..) => panic!("cannot resolve twice"),
        }
    }
}
