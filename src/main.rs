extern crate futures;
extern crate hyper;
extern crate tokio_core;

mod endpoint;
mod responder;

use std::sync::Arc;
use hyper::{Request, Method, Body};
use tokio_core::reactor::Core;

use endpoint::Endpoint;
use responder::Responder;

fn endpoint(_req: Arc<Request>) -> Option<Result<&'static str, ()>> {
    Some(Ok("hello"))
}

fn main() {
    let req = Request::<Body>::new(Method::Get, Default::default());
    let req = Arc::new(req);

    let mut core = Core::new().unwrap();
    let res = endpoint.apply(req.clone()).map(|f| {
        core.run(f).map(|r| r.respond())
    });

    println!("input: {:#?}", req);
    println!();
    println!("output: {:#?}", res);
}
