//! Definition of endpoints to parse query parameters

use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Query<T, E>(&'static str, PhantomData<fn() -> (T, E)>);

impl<T, E> Clone for Query<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> Copy for Query<T, E> {}

impl<T: FromStr, E> Endpoint for Query<T, E> {
    type Item = T;
    type Error = E;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        ctx.query(self.0)
            .ok_or(EndpointError::Skipped)
            .and_then(|s| s.parse().map_err(|_| EndpointError::TypeMismatch))
            .map(ok)
    }
}

/// Create an endpoint matches a query parameter named `name`
pub fn query<T: FromStr, E>(name: &'static str) -> Query<T, E> {
    Query(name, PhantomData)
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct QueryOpt<T, E>(&'static str, PhantomData<fn() -> (T, E)>);

impl<T, E> Clone for QueryOpt<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> Copy for QueryOpt<T, E> {}

impl<T: FromStr, E> Endpoint for QueryOpt<T, E> {
    type Item = Option<T>;
    type Error = E;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        ctx.query(self.0)
            .map(|s| s.parse().map_err(|_| EndpointError::TypeMismatch))
            .map_or(Ok(None), |s| s.map(Some))
            .map(ok)
    }
}

/// Create an endpoint matches a query parameter, which the value may not exist
pub fn query_opt<T: FromStr, E>(name: &'static str) -> QueryOpt<T, E> {
    QueryOpt(name, PhantomData)
}
