//! Helper functions for testing

#![allow(missing_docs)]

use std::io;
use hyper::Request;
use tokio_core::reactor::Core;

use endpoint::{Endpoint, EndpointContext};
use http::{self, HttpError};
use task::{Task, TaskContext};

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

    pub fn run(&mut self, request: Request) -> Option<Result<E::Item, Result<E::Error, HttpError>>> {
        let (mut request, body) = http::request::reconstruct(request);

        let task = {
            let mut ctx = EndpointContext::new(&request);
            try_opt!(self.endpoint.apply(&mut ctx))
        };

        let fut = {
            let mut ctx = TaskContext {
                request: &mut request,
                body: Some(body),
            };
            task.launch(&mut ctx)
        };

        Some(self.core.run(fut))
    }
}
