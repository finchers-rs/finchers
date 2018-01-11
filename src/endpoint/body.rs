use std::fmt;
use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext};
use http::{self, FromBody, FromBodyError};
use task;

#[allow(missing_docs)]
pub fn body<T, E>() -> Body<T, E>
where
    T: FromBody,
    E: From<FromBodyError<T::Error>>,
{
    Body {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Body<T, E> {
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Copy for Body<T, E> {}

impl<T, E> Clone for Body<T, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> fmt::Debug for Body<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl<T, E> Endpoint for Body<T, E>
where
    T: FromBody,
    E: From<FromBodyError<T::Error>>,
{
    type Item = T;
    type Error = E;
    type Task = task::body::Body<T, E>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match T::is_match(ctx.request()) {
            true => Some(task::body::Body {
                _marker: PhantomData,
            }),
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
pub struct BodyStream<E> {
    _marker: PhantomData<fn() -> E>,
}

impl<E> Copy for BodyStream<E> {}

impl<E> Clone for BodyStream<E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> fmt::Debug for BodyStream<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BodyStream").finish()
    }
}

impl<E> Endpoint for BodyStream<E> {
    type Item = http::Body;
    type Error = E;
    type Task = task::body::BodyStream<E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(task::body::BodyStream {
            _marker: PhantomData,
        })
    }
}
