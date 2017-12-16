#![allow(missing_docs)]

use std::marker::PhantomData;
use context::Context;
use task::TaskResult;
use super::{Endpoint, EndpointError};


pub fn empty<T, E>(error: EndpointError) -> Empty<T, E> {
    Empty {
        error,
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct Empty<T, E> {
    error: EndpointError,
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Endpoint for Empty<T, E> {
    type Item = T;
    type Error = E;
    type Task = TaskResult<T, E>;

    fn apply(&self, _: &mut Context) -> Result<Self::Task, EndpointError> {
        Err(self.error)
    }
}
