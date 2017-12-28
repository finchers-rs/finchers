use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext};
use http::{self, FromBody};
use task;

#[allow(missing_docs)]
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
