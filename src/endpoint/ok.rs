#![allow(missing_docs)]

use core::NeverReturn;
use super::{Endpoint, EndpointContext, Input};

pub fn ok<T: Clone>(x: T) -> EndpointOk<T> {
    EndpointOk { x }
}

#[derive(Debug, Clone, Copy)]
pub struct EndpointOk<T> {
    x: T,
}

impl<T: Clone> Endpoint for EndpointOk<T> {
    type Item = T;
    type Result = Result<T, NeverReturn>;

    fn apply(&self, _: &Input, _: &mut EndpointContext) -> Option<Self::Result> {
        Some(Ok(self.x.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::TestRunner;
    use http::Request;

    #[test]
    fn test_ok() {
        let endpoint = ok("Alice");
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::get("/").body(()).unwrap();
        let result: Option<Result<&str, _>> = runner.run(request);
        assert_eq!(result, Some(Ok("Alice")));
    }
}
