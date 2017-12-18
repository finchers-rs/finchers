use std::fmt;
use std::error::Error;
use std::marker::PhantomData;

use futures::Stream;
use hyper;
use request::{self, FromBody};
use task::{Poll, Task, TaskContext};

/// The type of a future returned from `Body::into_vec()`
#[derive(Debug)]
pub struct Body<T, E> {
    inner: Option<(request::Body, Vec<u8>)>,
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Default for Body<T, E> {
    fn default() -> Self {
        Body {
            inner: None,
            _marker: PhantomData,
        }
    }
}

impl<T, E> Body<T, E> {
    fn inner_mut(&mut self, ctx: &mut TaskContext) -> &mut (request::Body, Vec<u8>) {
        self.inner.get_or_insert_with(|| {
            let body = ctx.take_body().expect("cannot take the request body twice");
            (body, vec![])
        })
    }
}

impl<T, E> Task for Body<T, E>
where
    T: FromBody,
    E: From<BodyError<T::Error>>,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        loop {
            let (ref mut body, ref mut buf) = *self.inner_mut(ctx);
            match try_ready!(body.inner.poll().map_err(BodyError::Hyper)) {
                Some(item) => buf.extend_from_slice(&item),
                None => break,
            }
        }

        let (_, buf) = self.inner.take().expect("cannot resolve twice");

        match T::from_body(buf) {
            Ok(body) => Ok(body.into()),
            Err(err) => Err(BodyError::Parsing(err).into()),
        }
    }
}


/// The error type of `ParseBody<T>`
#[derive(Debug)]
pub enum BodyError<E> {
    /// Failure occurs when it receives the body stream
    Hyper(hyper::Error),
    /// Failure occurs when it parses the request body into `T`
    Parsing(E),
}

impl<E: fmt::Display> fmt::Display for BodyError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BodyError::Hyper(ref e) => e.fmt(f),
            BodyError::Parsing(ref e) => e.fmt(f),
        }
    }
}

impl<E: Error> Error for BodyError<E> {
    fn description(&self) -> &str {
        match *self {
            BodyError::Hyper(ref e) => e.description(),
            BodyError::Parsing(ref e) => e.description(),
        }
    }
}
