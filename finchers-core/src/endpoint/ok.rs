#![allow(missing_docs)]

use futures::future::{self, FutureResult};
use super::{Endpoint, EndpointContext};
use errors::Error;
use request::Input;

pub fn ok<T: Clone>(x: T) -> EndpointOk<T> {
    EndpointOk { x }
}

#[derive(Debug, Clone, Copy)]
pub struct EndpointOk<T> {
    x: T,
}

impl<T: Clone> Endpoint for EndpointOk<T> {
    type Item = T;
    type Future = FutureResult<T, Error>;

    fn apply(&self, _: &Input, _: &mut EndpointContext) -> Option<Self::Future> {
        Some(future::ok(self.x.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;
    use test::TestRunner;

    #[test]
    fn test_ok() {
        let endpoint = ok("Alice");
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::get("/").body(()).unwrap();
        assert_eq!(runner.run(request).ok(), Some("Alice"));
    }
}
