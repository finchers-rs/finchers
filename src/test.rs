//! Helper functions for testing

#![allow(missing_docs)]

use std::io;
use hyper::Request;
use tokio_core::reactor::Core;

use endpoint::Endpoint;
use service::EndpointExt;

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
