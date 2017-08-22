//! Definition of endpoints to parse query parameters

use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::Endpoint;
use errors::*;


#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Param<T>(&'static str, PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for Param<T> {
    type Item = T;
    type Future = FutureResult<T, FinchersError>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let value: T = match ctx.params.get(&*self.0).and_then(|s| s.parse().ok()) {
            Some(val) => val,
            None => return (ctx, Err(FinchersErrorKind::NotFound.into())),
        };
        (ctx, Ok(ok(value)))
    }
}

/// Create an endpoint which represents a query parameter
pub fn param<T: FromStr>(name: &'static str) -> Param<T> {
    Param(name, PhantomData)
}


#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct ParamOpt<T>(&'static str, PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for ParamOpt<T> {
    type Item = Option<T>;
    type Future = FutureResult<Option<T>, FinchersError>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let value: Option<T> = match ctx.params.get(&*self.0).map(|s| s.parse()) {
            Some(Ok(val)) => Some(val),
            Some(Err(_)) => return (ctx, Err(FinchersErrorKind::NotFound.into())),
            None => None,
        };
        (ctx, Ok(ok(value)))
    }
}

#[allow(missing_docs)]
pub fn param_opt<T: FromStr>(name: &'static str) -> ParamOpt<T> {
    ParamOpt(name, PhantomData)
}
