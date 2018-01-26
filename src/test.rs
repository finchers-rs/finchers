//! Helper functions for testing

#![allow(missing_docs)]

use std::io;
use http::Request;
use tokio_core::reactor::Core;
use endpoint::Endpoint;

#[derive(Debug)]
pub struct TestRunner<E: Endpoint> {
    endpoint: E,
    core: Core,
}

impl<E: Endpoint> TestRunner<E> {
    pub fn new(endpoint: E) -> io::Result<Self> {
        Ok(TestRunner {
            endpoint,
            core: Core::new()?,
        })
    }

    /// Apply an incoming HTTP request to the endpoint and return the result.
    ///
    /// # Panics
    /// This method will panic if an unexpected HTTP error will be occurred.
    pub fn run(&mut self, request: Request) -> Option<Result<E::Item, E::Error>> {
        self.endpoint.apply_request(request).map(|fut| {
            self.core
                .run(fut)
                .map_err(|e| e.expect("unexpected HTTP error"))
        })
    }
}

pub trait EndpointTestExt: Endpoint + sealed::Sealed {
    fn run(&self, request: Request) -> Option<Result<Self::Item, Self::Error>>;
}

impl<E: Endpoint> EndpointTestExt for E {
    fn run(&self, request: Request) -> Option<Result<Self::Item, Self::Error>> {
        let mut runner = TestRunner::new(self).unwrap();
        runner.run(request)
    }
}

mod sealed {
    use endpoint::Endpoint;

    pub trait Sealed {}

    impl<E: Endpoint> Sealed for E {}
}
