use std::fmt::Debug;
use hyper::Method;
use tokio_core::reactor::Core;

use context::Context;
use endpoint::{Endpoint,NewEndpoint};
use errors::EndpointResult;
use request::{Request, Body};


pub fn run_test<E: NewEndpoint>(new_endpoint: E, method: Method, uri: &str)
    -> EndpointResult<Result<E::Item, E::Error>>
where
    E::Item: Debug,
    E::Error: Debug,
{
    let endpoint = new_endpoint.new_endpoint();

    let req = Request {
        method,
        uri: uri.parse().unwrap(),
        headers: Default::default(),
        body: Some(Body::default()),
    };
    let ctx = Context::new(&req);

    let (_ctx, f) = endpoint.apply(ctx)?;

    let mut core = Core::new().unwrap();
    Ok(core.run(f))
}


