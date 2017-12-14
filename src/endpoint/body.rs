//! Definition of endpoints to parse request body

use std::marker::PhantomData;

use context::Context;
use endpoint::{Endpoint, EndpointError};
use request::{FromBody, ParseBody, ParseBodyError};


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
    type Future = ParseBody<T>;

    fn apply(&self, ctx: &mut Context) -> Result<Self::Future, EndpointError> {
        if !T::check_request(ctx.request()) {
            return Err(EndpointError::Skipped);
        }
        ctx.take_body()
            .ok_or_else(|| EndpointError::EmptyBody)
            .map(Into::into)
    }
}


/// Create an endpoint, represents the value of a request body
pub fn body<T: FromBody>() -> Body<T> {
    Body(PhantomData)
}
