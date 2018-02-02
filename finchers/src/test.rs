//! Helper functions for testing

#![allow(missing_docs)]

use std::io;
use tokio_core::reactor::Core;
use http::Request;

use body::BodyStream;
use endpoint::Endpoint;
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
    pub fn run<B>(&mut self, request: Request<B>) -> Option<Result<E::Item, Error>>
    where
        B: Into<BodyStream>,
    {
        self.endpoint
            .apply_input(::http::Request::from(request).into())
            .map(|fut| self.core.run(fut))
    }
}

pub trait EndpointTestExt: Endpoint + sealed::Sealed {
    fn run<B>(&self, request: Request<B>) -> Option<Result<Self::Item, Error>>
    where
        B: Into<BodyStream>;
}

impl<E: Endpoint> EndpointTestExt for E {
    fn run<B>(&self, request: Request<B>) -> Option<Result<Self::Item, Error>>
    where
        B: Into<BodyStream>,
    {
        let mut runner = TestRunner::new(self).unwrap();
        runner.run(request)
    }
}

mod sealed {
    use endpoint::Endpoint;

    pub trait Sealed {}

    impl<E: Endpoint> Sealed for E {}
}
