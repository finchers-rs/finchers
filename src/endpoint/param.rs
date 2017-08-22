//! Definition of endpoints to parse query parameters

use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::Endpoint;
use errors::*;


#[allow(missing_docs)]
pub struct Param<T>(Cow<'static, str>, PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for Param<T> {
    type Item = T;
    type Future = FutureResult<T, FinchersError>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let value: T = match ctx.params.get(&self.0).and_then(|s| s.parse().ok()) {
            Some(val) => val,
            None => return (ctx, Err(FinchersErrorKind::NotFound.into())),
        };
        (ctx, Ok(ok(value)))
    }
}

/// Create an endpoint which represents a query parameter
pub fn param<T: FromStr>(name: &'static str) -> Param<T> {
    Param(name.into(), PhantomData)
}
