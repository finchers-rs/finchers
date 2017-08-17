extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;

pub mod endpoint;
pub mod responder;
pub mod input;

use std::sync::Arc;
use hyper::Get;
use tokio_core::reactor::Core;

use endpoint::{Endpoint, param};
use input::Input;
use responder::Responder;

fn main() {
    let endpoint = param("hello");

    let input = Arc::new(Input::new(Get, "/?hello=world"));
    let output = endpoint.apply(input.clone()).map(|f| {
        let mut core = Core::new().unwrap();
        core.run(f).map(|r| r.respond())
    });

    println!("input: {:#?}", input);
    println!();
    println!("output: {:#?}", output);
    println!(
        "body: {:#?}",
        output.as_ref().map(
            |res| res.as_ref().map(|res| res.body_ref()),
        )
    );
}
