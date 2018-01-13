use std::fmt;
use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext};
use http::{self, FromBody, FromBodyError};
use super::task;

#[allow(missing_docs)]
pub fn body<T: FromBody>() -> Body<T> {
    Body {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
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
    type Error = FromBodyError<T::Error>;
    type Task = task::body::Body<T>;

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
