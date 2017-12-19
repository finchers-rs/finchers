use std::marker::PhantomData;
use endpoint::{Endpoint, EndpointContext, EndpointError};
use task::{self, TaskResult};


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
    type Task = TaskResult<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        Ok(task::ok(self.x.clone()))
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
    type Task = TaskResult<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        Ok(task::err(self.x.clone()))
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
    type Task = TaskResult<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        Ok(task::result(self.x.clone()))
    }
}
