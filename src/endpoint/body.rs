use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext};
use http::{self, FromBody};
use task;

#[allow(missing_docs)]
pub fn body<T, E>() -> Body<T, E>
where
    T: FromBody,
    E: From<T::Error>,
{
    Body {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Body<T, E> {
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Endpoint for Body<T, E>
where
    T: FromBody,
    E: From<T::Error>,
{
    type Item = T;
    type Error = E;
    type Task = task::body::Body<T, E>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match T::is_match(ctx.request()) {
            true => Some(task::body::Body::default()),
            false => None,
        }
    }
}

#[allow(missing_docs)]
pub fn body_stream<E>() -> BodyStream<E> {
    BodyStream {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct BodyStream<E> {
    _marker: PhantomData<fn() -> E>,
}

impl<E> Endpoint for BodyStream<E> {
    type Item = http::Body;
    type Error = E;
    type Task = task::body::BodyStream<E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(task::body::BodyStream::default())
    }
}
