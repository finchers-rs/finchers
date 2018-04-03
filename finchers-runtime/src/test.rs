//! Helper functions for testing

#![allow(missing_docs)]

use std::io;
use tokio_core::reactor::Core;
use http::Request;

use finchers_core::endpoint::{Endpoint, Outcome};
use finchers_core::request::body::BodyStream;

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
    pub fn run<B>(&mut self, request: Request<B>) -> Outcome<E::Item>
    where
        B: Into<BodyStream>,
    {
        let outcome = self.endpoint
            .apply_input(::http::Request::from(request).into());
        self.core.run(outcome).unwrap()
    }
}

pub trait EndpointTestExt: Endpoint + sealed::Sealed {
    fn run<B>(&self, request: Request<B>) -> Outcome<Self::Item>
    where
        B: Into<BodyStream>;
}

impl<E: Endpoint> EndpointTestExt for E {
    fn run<B>(&self, request: Request<B>) -> Outcome<Self::Item>
    where
        B: Into<BodyStream>,
    {
        let mut runner = TestRunner::new(self).unwrap();
        runner.run(request)
    }
}

mod sealed {
    use finchers_core::endpoint::Endpoint;

    pub trait Sealed {}

    impl<E: Endpoint> Sealed for E {}
}
