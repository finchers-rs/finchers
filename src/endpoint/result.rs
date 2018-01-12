#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;
use super::{Endpoint, EndpointContext};

pub fn ok<T: Clone, E>(x: T) -> EndpointOk<T, E> {
    EndpointOk {
        x,
        _marker: PhantomData,
    }
}

pub struct EndpointOk<T, E> {
    x: T,
    _marker: PhantomData<fn() -> E>,
}

impl<T: Clone, E> Clone for EndpointOk<T, E> {
    fn clone(&self) -> Self {
        EndpointOk {
            x: self.x.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: fmt::Debug, E> fmt::Debug for EndpointOk<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("EndpointOk").field(&self.x).finish()
    }
}

impl<T: Clone, E> Endpoint for EndpointOk<T, E> {
    type Item = T;
    type Error = E;
    type Task = Result<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(Ok(self.x.clone()))
    }
}

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

impl<T, E: Clone> Endpoint for EndpointErr<T, E> {
    type Item = T;
    type Error = E;
    type Task = Result<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(Err(self.x.clone()))
    }
}

pub fn result<T: Clone, E: Clone>(x: Result<T, E>) -> EndpointResult<T, E> {
    EndpointResult { x }
}

#[derive(Clone)]
pub struct EndpointResult<T, E> {
    x: Result<T, E>,
}

impl<T: fmt::Debug, E: fmt::Debug> fmt::Debug for EndpointResult<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("EndpointResult").field(&self.x).finish()
    }
}

impl<T: Clone, E: Clone> Endpoint for EndpointResult<T, E> {
    type Item = T;
    type Error = E;
    type Task = Result<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(self.x.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::TestRunner;
    use hyper::{Method, Request};

    #[test]
    fn test_ok() {
        let endpoint = ok("Alice");
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/".parse().unwrap());
        let result: Option<Result<&str, ()>> = runner.run(request);
        assert_eq!(result, Some(Ok("Alice")));
    }

    #[test]
    fn test_err() {
        let endpoint = err("Alice");
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/".parse().unwrap());
        let result: Option<Result<(), &str>> = runner.run(request);
        assert_eq!(result, Some(Err("Alice")));
    }

    #[test]
    fn test_result() {
        let endpoint = result(Ok("Alice"));
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/".parse().unwrap());
        let result: Option<Result<&str, ()>> = runner.run(request);
        assert_eq!(result, Some(Ok("Alice")));
    }
}
