extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;

pub mod endpoint;
pub mod responder;
pub mod input;

use hyper::Get;
use tokio_core::reactor::Core;

use endpoint::{Endpoint, param};
use input::Input;
use responder::Responder;

fn main() {
    let endpoint = param("hello");

    let input = Input::new(Get, "/?hello=world");
    println!("input: {:#?}", input);
    println!();

    let output = endpoint.apply(input).map(|f| {
        let mut core = Core::new().unwrap();
        core.run(f).map(|r| r.respond())
    });
    println!("output: {:#?}", output);
    println!(
        "body: {:#?}",
        output.as_ref().map(
            |res| res.as_ref().map(|res| res.body_ref()),
        )
    );
}
