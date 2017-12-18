#![allow(missing_docs)]

use std::marker::PhantomData;
use task::TaskResult;
use super::{Endpoint, EndpointContext, EndpointError};


pub fn reject<T, E>(error: EndpointError) -> Reject<T, E> {
    Reject {
        error,
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct Reject<T, E> {
    error: EndpointError,
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Endpoint for Reject<T, E> {
    type Item = T;
    type Error = E;
    type Task = TaskResult<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        Err(self.error)
    }
}
