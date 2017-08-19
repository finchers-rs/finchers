use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::Endpoint;
use errors::*;
use request::Body;


pub struct Param<T>(Cow<'static, str>, PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for Param<T> {
    type Item = T;
    type Future = FutureResult<T, FinchersError>;

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let value: T = match ctx.params.get(&self.0).and_then(|s| s.parse().ok()) {
            Some(val) => val,
            None => return Err((FinchersErrorKind::Routing.into(), body)),
        };
        Ok((ctx, body, ok(value)))
    }
}

pub fn param<T: FromStr>(name: &'static str) -> Param<T> {
    Param(name.into(), PhantomData)
}
