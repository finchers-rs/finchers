use std::marker::PhantomData;
use futures::{Future, Poll, Stream};
use http::{self, FromBody};
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
    E: From<http::Error> + From<T::Error>,
{
    type Item = T;
    type Error = E;
    type Future = BodyFuture<T, E>;
    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        let body = ctx.take_body().expect("cannot take the request body twice");
        BodyFuture {
            inner: Some((body, vec![])),
            _marker: PhantomData,
        }
    }
}


#[derive(Debug)]
pub struct BodyFuture<T, E> {
    inner: Option<(http::Body, Vec<u8>)>,
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Future for BodyFuture<T, E>
where
    T: FromBody,
    E: From<http::Error> + From<T::Error>,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let (ref mut body, ref mut buf) = *self.inner.as_mut().expect("cannot resolve twice");
            match try_ready!(body.poll()) {
                Some(item) => buf.extend_from_slice(&item),
                None => break,
            }
        }

        let (_, buf) = self.inner.take().expect("cannot resolve twice");
        let body = T::from_body(buf)?;
        Ok(body.into())
    }
}
