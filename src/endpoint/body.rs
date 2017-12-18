//! Definition of endpoints to parse request body

use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext, EndpointError};
use request::FromBody;
use task::{ParseBody, ParseBodyError};


/// Create an endpoint, represents the value of a request body
pub fn body<T: FromBody>() -> Body<T> {
    Body {
        _marker: PhantomData,
    }
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Body<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Body<T> {}

impl<T> Clone for Body<T> {
    fn clone(&self) -> Body<T> {
        *self
    }
}

impl<T: FromBody> Endpoint for Body<T> {
    type Item = T;
    type Error = ParseBodyError<T::Error>;
    type Task = ParseBody<T>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        match T::check_request(ctx.request()) {
            true => Ok(ParseBody::default()),
            false => Err(EndpointError::Skipped),
        }
    }
}
