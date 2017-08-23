//! Definition of endpoints to parse query parameters

use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};
use errors::*;


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Query<T>(&'static str, PhantomData<fn(T) -> T>);

impl<T> Clone for Query<T> {
    fn clone(&self) -> Self {
        Query(self.0, PhantomData)
    }
}

impl<T> Copy for Query<T> {}

impl<T: FromStr> Endpoint for Query<T> {
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
pub fn query<T: FromStr>(name: &'static str) -> Query<T> {
    Query(name, PhantomData)
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct QueryOpt<T>(&'static str, PhantomData<fn(T) -> T>);

impl<T> Clone for QueryOpt<T> {
    fn clone(&self) -> QueryOpt<T> {
        QueryOpt(self.0, PhantomData)
    }
}

impl<T> Copy for QueryOpt<T> {}

impl<T: FromStr> Endpoint for QueryOpt<T> {
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
pub fn query_opt<T: FromStr>(name: &'static str) -> QueryOpt<T> {
    QueryOpt(name, PhantomData)
}
