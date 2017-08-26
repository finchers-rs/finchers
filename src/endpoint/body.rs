//! Definition of endpoints to parse request body

use std::marker::PhantomData;
use futures::Future;
use futures::future::{err, ok, AndThen, Flatten, FutureResult};

use hyper::header::ContentType;
use hyper::mime::TEXT_PLAIN_UTF_8;

use serde::Deserialize;

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};
use errors::*;
use request::{self, IntoVec, Request};
use json::Json;


/// A trait represents the conversion from `Body`.
pub trait FromBody: Sized {
    /// A future returned from `from_body()`
    type Future: Future<Item = Self, Error = FinchersError>;

    /// Convert the content of `body` to its type
    fn from_body(body: request::Body, req: &Request) -> Self::Future;
}


impl FromBody for String {
    type Future = Flatten<
        FutureResult<AndThen<IntoVec, FinchersResult<String>, fn(Vec<u8>) -> FinchersResult<String>>, FinchersError>,
    >;

    fn from_body(body: request::Body, req: &Request) -> Self::Future {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == TEXT_PLAIN_UTF_8 => (),
            _ => return err(FinchersErrorKind::BadRequest.into()).flatten(),
        }

        ok(body.into_vec().and_then(
            (|body| String::from_utf8(body).map_err(|_| FinchersErrorKind::BadRequest.into())) as
                fn(Vec<u8>) -> FinchersResult<String>,
        )).flatten()
    }
}


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
    type Error = FinchersError;
    type Future = T::Future;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        ctx.take_body()
            .ok_or_else(|| EndpointError::EmptyBody)
            .map(|body| T::from_body(body, ctx.request()))
    }
}


/// Create an endpoint, represents the value of a request body
pub fn body<T: FromBody>() -> Body<T> {
    Body(PhantomData)
}

/// Equivalent to `body::<Json<T>>()`
pub fn json_body<T>() -> Body<Json<T>>
where
    for<'de> T: Deserialize<'de>,
{
    Body(PhantomData)
}
