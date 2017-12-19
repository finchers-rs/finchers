use std::marker::PhantomData;
use std::str::FromStr;

use endpoint::{Endpoint, EndpointContext, EndpointError};
use task::{ok, TaskResult};


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
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        ctx.query(self.0)
            .ok_or(EndpointError::Skipped)
            .and_then(|s| s.parse().map_err(|_| EndpointError::TypeMismatch))
            .map(ok)
    }
}


pub fn query<T: FromStr, E>(name: &'static str) -> Query<T, E> {
    Query(name, PhantomData)
}



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
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        ctx.query(self.0)
            .map(|s| s.parse().map_err(|_| EndpointError::TypeMismatch))
            .map_or(Ok(None), |s| s.map(Some))
            .map(ok)
    }
}


pub fn query_opt<T: FromStr, E>(name: &'static str) -> QueryOpt<T, E> {
    QueryOpt(name, PhantomData)
}
