use std::marker::PhantomData;
use futures::{Future, Stream, Poll, BoxFuture};
use hyper::StatusCode;
use hyper::header::ContentType;
use hyper::mime::TEXT_PLAIN_UTF_8;

use context::Context;
use endpoint::Endpoint;
use errors::EndpointResult;
use request::{Request, Body};


pub trait FromBody: Sized {
    type Future: Future<Item = Self, Error = StatusCode>;

    fn from_body(body: Body, req: &Request) -> Self::Future;
}

impl FromBody for String {
    type Future = FromBodyFuture<String>;

    fn from_body(body: Body, req: &Request) -> Self::Future {
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

pub enum FromBodyFuture<T> {
    WrongMediaType,
    Parsed(BoxFuture<T, ()>),
}

impl<T> Future for FromBodyFuture<T> {
    type Item = T;
    type Error = StatusCode;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            FromBodyFuture::WrongMediaType => Err(StatusCode::BadRequest),
            FromBodyFuture::Parsed(ref mut f) => f.poll().map_err(|_| StatusCode::BadRequest),
        }
    }
}



pub struct TakeBody<T>(PhantomData<fn(T) -> T>);

impl<T: FromBody> Endpoint for TakeBody<T> {
    type Item = T;
    type Future = T::Future;

    fn apply<'r>(self, ctx: Context<'r>, mut body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        if let Some(body) = body.take() {
            let value = T::from_body(body, &ctx.request);
            Ok((ctx, None, value))
        } else {
            Err(("cannot take body twice".into(), None))
        }
    }
}


pub fn take_body<T: FromBody>() -> TakeBody<T> {
    TakeBody(PhantomData)
}
