use std::fmt::Debug;
use hyper::{Method, StatusCode};
use tokio_core::reactor::Core;

use context::Context;
use endpoint::{Endpoint, NewEndpoint};
use errors::EndpointResult;
use request::Request;


pub fn run_test<E: NewEndpoint>(
    new_endpoint: E,
    method: Method,
    uri: &str,
) -> EndpointResult<Result<E::Item, StatusCode>>
where
    E::Item: Debug,
{
    let endpoint = new_endpoint.new_endpoint();

    let req = Request::new(method, uri).expect("invalid URI");
    let ctx = Context::new(&req);

    let (_ctx, f) = endpoint.apply(ctx)?;

    let mut core = Core::new().unwrap();
    Ok(core.run(f))
}
