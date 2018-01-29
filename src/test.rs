//! Helper functions for testing

#![allow(missing_docs)]

use std::io;
use tokio_core::reactor::Core;
use endpoint::{Endpoint, Input};
use errors::Error;

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
    pub fn run<I: Into<Input>>(&mut self, input: I) -> Option<Result<E::Item, Error>> {
        self.endpoint
            .apply_request(input)
            .map(|fut| self.core.run(fut))
    }
}

pub trait EndpointTestExt: Endpoint + sealed::Sealed {
    fn run<I: Into<Input>>(&self, input: I) -> Option<Result<Self::Item, Error>>;
}

impl<E: Endpoint> EndpointTestExt for E {
    fn run<I: Into<Input>>(&self, input: I) -> Option<Result<Self::Item, Error>> {
        let mut runner = TestRunner::new(self).unwrap();
        runner.run(input)
    }
}

mod sealed {
    use endpoint::Endpoint;

    pub trait Sealed {}

    impl<E: Endpoint> Sealed for E {}
}
