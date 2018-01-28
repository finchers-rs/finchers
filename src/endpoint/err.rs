#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;
use errors::HttpError;
use endpoint::{Endpoint, EndpointContext};

pub fn err<T, E: Clone>(x: E) -> EndpointErr<T, E> {
    EndpointErr {
        x,
        _marker: PhantomData,
    }
}

pub struct EndpointErr<T, E> {
    x: E,
    _marker: PhantomData<fn() -> T>,
}

impl<T, E: Clone> Clone for EndpointErr<T, E> {
    fn clone(&self) -> Self {
        EndpointErr {
            x: self.x.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T, E: fmt::Debug> fmt::Debug for EndpointErr<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("EndpointErr").field(&self.x).finish()
    }
}

impl<T, E: Clone> Endpoint for EndpointErr<T, E>
where
    E: HttpError,
{
    type Item = T;
    type Error = E;
    type Result = Result<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(Err(self.x.clone()))
    }
}
