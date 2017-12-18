use std::marker::PhantomData;
use futures::Stream;
use hyper;
use request::{Body, FromBody};
use task::{Poll, Task, TaskContext};

/// The type of a future returned from `Body::into_vec()`
#[derive(Debug)]
pub struct ParseBody<T> {
    inner: Option<(Body, Vec<u8>)>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromBody> Default for ParseBody<T> {
    fn default() -> Self {
        ParseBody {
            inner: None,
            _marker: PhantomData,
        }
    }
}

impl<T: FromBody> ParseBody<T> {
    fn inner_mut(&mut self, ctx: &mut TaskContext) -> &mut (Body, Vec<u8>) {
        self.inner.get_or_insert_with(|| {
            let body = ctx.take_body().expect("cannot take the request body twice");
            (body, vec![])
        })
    }
}

impl<T: FromBody> Task for ParseBody<T> {
    type Item = T;
    type Error = ParseBodyError<T::Error>;

    fn poll(&mut self, ctx: &mut TaskContext) -> Poll<Self::Item, Self::Error> {
        loop {
            let (ref mut body, ref mut buf) = *self.inner_mut(ctx);
            match try_ready!(body.inner.poll()) {
                Some(item) => buf.extend_from_slice(&item),
                None => break,
            }
        }

        let (_, buf) = self.inner
            .take()
            .expect("The buffer has been already taken");
        T::from_body(buf)
            .map(Into::into)
            .map_err(ParseBodyError::Parse)
    }
}


/// The error type of `ParseBody<T>`
#[derive(Debug)]
pub enum ParseBodyError<E> {
    /// Failure occurs when it receives the body stream
    Hyper(hyper::Error),
    /// Failure occurs when it parses the request body into `T`
    Parse(E),
}

impl<T> From<hyper::Error> for ParseBodyError<T> {
    fn from(err: hyper::Error) -> Self {
        ParseBodyError::Hyper(err)
    }
}
