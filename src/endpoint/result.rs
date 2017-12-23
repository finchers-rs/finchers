#![allow(missing_docs)]

use std::marker::PhantomData;
use futures::future::{self, FutureResult};
use task::TaskFuture;
use super::{Endpoint, EndpointContext};


pub fn ok<T: Clone, E>(x: T) -> EndpointOk<T, E> {
    EndpointOk {
        x,
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct EndpointOk<T: Clone, E> {
    x: T,
    _marker: PhantomData<fn() -> E>,
}

impl<T: Clone, E> Endpoint for EndpointOk<T, E> {
    type Item = T;
    type Error = E;
    type Task = TaskFuture<FutureResult<T, E>>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(future::ok(self.x.clone()).into())
    }
}


pub fn err<T, E: Clone>(x: E) -> EndpointErr<T, E> {
    EndpointErr {
        x,
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct EndpointErr<T, E: Clone> {
    x: E,
    _marker: PhantomData<fn() -> T>,
}

impl<T, E: Clone> Endpoint for EndpointErr<T, E> {
    type Item = T;
    type Error = E;
    type Task = TaskFuture<FutureResult<T, E>>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(future::err(self.x.clone()).into())
    }
}



pub fn result<T: Clone, E: Clone>(x: Result<T, E>) -> EndpointResult<T, E> {
    EndpointResult { x }
}

#[derive(Debug)]
pub struct EndpointResult<T: Clone, E: Clone> {
    x: Result<T, E>,
}

impl<T: Clone, E: Clone> Endpoint for EndpointResult<T, E> {
    type Item = T;
    type Error = E;
    type Task = TaskFuture<FutureResult<T, E>>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(future::result(self.x.clone()).into())
    }
}
