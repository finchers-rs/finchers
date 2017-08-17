extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;

pub mod endpoint;
pub mod responder;

use std::sync::Arc;
use hyper::{Request, Method, Body};
use tokio_core::reactor::Core;

use endpoint::{Endpoint, param};
use responder::Responder;

fn main() {
    let endpoint = param("hello");

    let uri = "/?hello=world".parse().unwrap();
    let req = Request::<Body>::new(Method::Get, uri);
    let req = Arc::new(req);

    let res = endpoint.apply(req.clone()).map(|f| {
        let mut core = Core::new().unwrap();
        core.run(f).map(|r| r.respond())
    });

    println!("request: {:#?}", req);
    println!();
    println!("response: {:#?}", res);
    println!(
        "response body: {:#?}",
        res.as_ref().map(
            |res| res.as_ref().map(|res| res.body_ref()),
        )
    );
}
