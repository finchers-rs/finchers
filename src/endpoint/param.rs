//! Definition of endpoints to parse query parameters

use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};
use errors::*;


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Param<T>(&'static str, PhantomData<fn(T) -> T>);

impl<T> Clone for Param<T> {
    fn clone(&self) -> Param<T> {
        Param(self.0, PhantomData)
    }
}

impl<T> Copy for Param<T> {}

impl<T: FromStr> Endpoint for Param<T> {
    type Item = T;
    type Future = FutureResult<T, FinchersError>;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        match ctx.params.get(&*self.0).and_then(|s| s.parse().ok()) {
            Some(val) => Ok(ok(val)),
            None => return Err(EndpointError::TypeMismatch),
        }
    }
}

/// Create an endpoint matches a query parameter named `name`
pub fn param<T: FromStr>(name: &'static str) -> Param<T> {
    Param(name, PhantomData)
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct ParamOpt<T>(&'static str, PhantomData<fn(T) -> T>);

impl<T> Clone for ParamOpt<T> {
    fn clone(&self) -> ParamOpt<T> {
        ParamOpt(self.0, PhantomData)
    }
}

impl<T> Copy for ParamOpt<T> {}

impl<T: FromStr> Endpoint for ParamOpt<T> {
    type Item = Option<T>;
    type Future = FutureResult<Option<T>, FinchersError>;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        match ctx.params.get(&*self.0).map(|s| s.parse()) {
            Some(Ok(val)) => Ok(ok(Some(val))),
            None => Ok(ok(None)),
            Some(Err(_)) => Err(EndpointError::TypeMismatch),
        }
    }
}

/// Create an endpoint matches a query parameter, which the value may not exist
pub fn param_opt<T: FromStr>(name: &'static str) -> ParamOpt<T> {
    ParamOpt(name, PhantomData)
}
