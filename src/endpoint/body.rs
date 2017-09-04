//! Definition of endpoints to parse request body

use std::marker::PhantomData;
use serde::de::DeserializeOwned;

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};
use request::FromBody;
use json::Json;


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
    type Error = T::Error;
    type Future = T::Future;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        if !T::check_request(ctx.request()) {
            return Err(EndpointError::Skipped);
        }
        ctx.take_body()
            .ok_or_else(|| EndpointError::EmptyBody)
            .map(T::from_body)
    }
}


/// Create an endpoint, represents the value of a request body
pub fn body<T: FromBody>() -> Body<T> {
    Body(PhantomData)
}

/// Equivalent to `body::<Json<T>>()`
pub fn json_body<T: DeserializeOwned>() -> Body<Json<T>> {
    Body(PhantomData)
}
