//! Definition of endpoints to parse request body

use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext};
use http::{self, FromBody};
use task;

/// Create an endpoint, represents the value of a request body
pub fn body<T, E>() -> Body<T, E>
where
    T: FromBody,
    E: From<http::HttpError> + From<T::Error>,
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

impl<T, E> Copy for Body<T, E> {}

impl<T, E> Clone for Body<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> Endpoint for Body<T, E>
where
    T: FromBody,
    E: From<http::HttpError> + From<T::Error>,
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
