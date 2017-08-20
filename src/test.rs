//! Helper functions for testing

use std::cell::RefCell;
use hyper::Method;
use tokio_core::reactor::Core;

use context::Context;
use endpoint::{Endpoint, NewEndpoint};
use errors::*;
use request::{Request, Body};


/// Invoke given endpoint factory and return its result
pub fn run_test<E: NewEndpoint>(new_endpoint: E, method: Method, uri: &str) -> Result<E::Item, FinchersError> {
    let endpoint = new_endpoint.new_endpoint();

    let req = Request::new(method, uri).expect("invalid URI");
    let body = RefCell::new(Some(Body::default()));
    let ctx = Context::new(&req, &body);

    let (_ctx, f) = endpoint.apply(ctx);
    let f = f?;

    let mut core = Core::new().unwrap();
    core.run(f)
}
