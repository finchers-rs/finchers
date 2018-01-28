#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;
use super::{Endpoint, EndpointContext};
use errors::HttpError;

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

impl<T: Clone, E: HttpError> Endpoint for EndpointOk<T, E> {
    type Item = T;
    type Error = E;
    type Result = Result<T, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(Ok(self.x.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::TestRunner;
    use errors::NeverReturn;
    use hyper::{Method, Request};

    #[test]
    fn test_ok() {
        let endpoint = ok("Alice");
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/".parse().unwrap());
        let result: Option<Result<&str, NeverReturn>> = runner.run(request);
        assert_eq!(result, Some(Ok("Alice")));
    }
}
