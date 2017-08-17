#![feature(conservative_impl_trait)]

#[macro_use]
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;

pub mod endpoint;
pub mod responder;
pub mod input;

use futures::{Future, Stream};
use hyper::Get;
use tokio_core::reactor::Core;

use endpoint::Endpoint;
use endpoint::param::param;
use endpoint::path::{path, path_end, value};

use input::Input;
use responder::Responder;

/// "/foo/:id/bar?hello=<hello>"
#[cfg_attr(rustfmt, rustfmt_skip)]
fn endpoint() -> impl Endpoint<Item = (u64, String)> + std::fmt::Debug {
    path("foo",
        value::<u64, _>(
            path("bar",
                 path_end(param("hello")))))
}

fn main() {
    let endpoint = endpoint();
    println!("endpoint: {:#?}", endpoint);

    let input = Input::new(Get, "/foo/42/bar?hello=world");
    println!("input: {:#?}", input);
    println!();

    if let Ok(f) = endpoint.apply(input) {
        let f = f.map(|(id, hello)| format!("({}, {})", id, hello)).map(
            |r| {
                r.respond()
            },
        );

        let mut core = Core::new().unwrap();
        let output = core.run(f);

        match output {
            Ok(response) => {
                println!("success: {:#?}", response);
                let body = core.run(
                    response
                        .body()
                        .map_err(|_| ())
                        .fold(Vec::new(), |mut body, chunk| {
                            body.extend_from_slice(&chunk);
                            Ok(body)
                        })
                        .and_then(|body| String::from_utf8(body).map_err(|_| ())),
                );
                println!("..with body: {:?}", body);
            }
            Err(_err) => {}//eprintln!("failed with: {:?}", err),
        }
    } else {
        eprintln!("no route");
    }
}
