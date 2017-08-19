use std::marker::PhantomData;
use futures::{Future, Stream, Poll, BoxFuture};
use hyper::StatusCode;
use hyper::header::ContentType;
use hyper::mime::TEXT_PLAIN_UTF_8;

use context::Context;
use endpoint::Endpoint;
use errors::*;
use request::{self, Request};


pub trait FromBody: Sized {
    type Future: Future<Item = Self, Error = FinchersError>;

    fn from_body(body: request::Body, req: &Request) -> Self::Future;
}

impl FromBody for String {
    type Future = FromBodyFuture<BoxFuture<String, ()>>;

    fn from_body(body: request::Body, req: &Request) -> Self::Future {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == TEXT_PLAIN_UTF_8 => (),
            _ => return FromBodyFuture::WrongMediaType,
        }

        FromBodyFuture::Parsed(
            body.map_err(|_| ())
                .fold(Vec::new(), |mut body, chunk| -> Result<Vec<u8>, ()> {
                    body.extend_from_slice(&chunk);
                    Ok(body)
                })
                .and_then(|body| String::from_utf8(body).map_err(|_| ()))
                .boxed(),
        )
    }
}

pub enum FromBodyFuture<F> {
    WrongMediaType,
    Parsed(F),
}

impl<F: Future> Future for FromBodyFuture<F> {
    type Item = F::Item;
    type Error = FinchersError;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            FromBodyFuture::WrongMediaType => Err(FinchersErrorKind::Status(StatusCode::BadRequest).into()),
            FromBodyFuture::Parsed(ref mut f) => {
                f.poll().map_err(|_| {
                    FinchersErrorKind::Status(StatusCode::BadRequest).into()
                })
            }
        }
    }
}



pub struct Body<T>(PhantomData<fn(T) -> T>);

impl<T: FromBody> Endpoint for Body<T> {
    type Item = T;
    type Future = T::Future;

    fn apply<'r>(self, ctx: Context<'r>, mut body: Option<request::Body>) -> EndpointResult<'r, Self::Future> {
        if let Some(body) = body.take() {
            let value = T::from_body(body, &ctx.request);
            Ok((ctx, None, value))
        } else {
            Err(("cannot take body twice".into(), None))
        }
    }
}


pub fn body<T: FromBody>() -> Body<T> {
    Body(PhantomData)
}
