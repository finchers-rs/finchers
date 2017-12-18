//! Definition of endpoints to parse request body

use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext, EndpointError};
use request::FromBody;
use task;


/// Create an endpoint, represents the value of a request body
pub fn body<T, E>() -> Body<T, E>
where
    T: FromBody,
    E: From<task::BodyError<T::Error>>,
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
    E: From<task::BodyError<T::Error>>,
{
    type Item = T;
    type Error = E;
    type Task = task::Body<T, E>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        match T::check_request(ctx.request()) {
            true => Ok(task::Body::default()),
            false => Err(EndpointError::Skipped),
        }
    }
}
