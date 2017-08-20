//! Definition of endpoints to parse request body

use std::marker::PhantomData;
use futures::{Future, Stream, BoxFuture};
use hyper::StatusCode;
use hyper::header::ContentType;
use hyper::mime::TEXT_PLAIN_UTF_8;

use context::Context;
use endpoint::Endpoint;
use errors::*;
use request::{self, Request};


/// A trait represents the conversion from `Body`.
pub trait FromBody: Sized {
    /// A future returned from `from_body()`
    type Future: Future<Item = Self, Error = FinchersError>;

    /// Convert the content of `body` to its type
    fn from_body(body: request::Body, req: &Request) -> FinchersResult<Self::Future>;
}

impl FromBody for String {
    type Future = BoxFuture<String, FinchersError>;

    fn from_body(body: request::Body, req: &Request) -> FinchersResult<Self::Future> {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == TEXT_PLAIN_UTF_8 => (),
            _ => return Err(FinchersErrorKind::Status(StatusCode::BadRequest).into()),
        }

        Ok(
            body.map_err(|err| FinchersErrorKind::ServerError(Box::new(err)).into())
                .fold(Vec::new(), |mut body,
                 chunk|
                 -> Result<Vec<u8>, FinchersError> {
                    body.extend_from_slice(&chunk);
                    Ok(body)
                })
                .and_then(|body| {
                    String::from_utf8(body).map_err(|_| FinchersErrorKind::Status(StatusCode::BadRequest).into())
                })
                .boxed(),
        )
    }
}


#[allow(missing_docs)]
pub struct Body<T>(PhantomData<fn(T) -> T>);

impl<T: FromBody> Endpoint for Body<T> {
    type Item = T;
    type Future = T::Future;

    fn apply<'r, 'b>(self, mut ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let result = if let Some(body) = ctx.take_body() {
            T::from_body(body, &ctx.request)
        } else {
            Err("cannot take body twice".into())
        };
        (ctx, result)
    }
}


/// Create a combinator to take a request body typed to `T` from the context
pub fn body<T: FromBody>() -> Body<T> {
    Body(PhantomData)
}
