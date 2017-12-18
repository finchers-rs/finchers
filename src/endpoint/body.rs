//! Definition of endpoints to parse request body

use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext, EndpointError};
use request::FromBody;
use task::{ParseBody, ParseBodyError};


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Body<T>(PhantomData<fn(T) -> T>);

impl<T> Clone for Body<T> {
    fn clone(&self) -> Body<T> {
        Body(PhantomData)
    }
}

impl<T> Copy for Body<T> {}

impl<T: FromBody> Endpoint for Body<T> {
    type Item = T;
    type Error = ParseBodyError<T::Error>;
    type Task = ParseBody<T>;

    #[allow(deprecated)]
    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        if !T::check_request(ctx.request()) {
            return Err(EndpointError::Skipped);
        }
        ctx.take_body()
            .ok_or_else(|| EndpointError::EmptyBody)
            .map(|body| ParseBody::new(body.inner))
    }
}


/// Create an endpoint, represents the value of a request body
pub fn body<T: FromBody>() -> Body<T> {
    Body(PhantomData)
}
