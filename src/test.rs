use hyper::Method;
use tokio_core::reactor::Core;

use context::Context;
use endpoint::{Endpoint, NewEndpoint};
use errors::*;
use request::{Request, Body};


pub fn run_test<E: NewEndpoint>(new_endpoint: E, method: Method, uri: &str) -> Result<E::Item, FinchersError> {
    let endpoint = new_endpoint.new_endpoint();

    let req = Request::new(method, uri).expect("invalid URI");
    let body = Body::default();
    let ctx = Context::new(&req);

    let (_ctx, _body, f) = endpoint.apply(ctx, Some(body)).map_err(|(err, _)| err)?;

    let mut core = Core::new().unwrap();
    core.run(f)
}
