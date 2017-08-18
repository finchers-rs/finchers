#![feature(conservative_impl_trait)]

extern crate finch;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use finch::combinator::{path, path_seq};
use finch::endpoint::{Context, Endpoint};
use finch::request::Request;

use hyper::Get;
use tokio_core::reactor::Core;

fn main() {
    let req = Request {
        method: Get,
        uri: "/foo/bar/42/baz/foo/bar/".parse().unwrap(),
        headers: Default::default(),
    };
    let mut ctx = Context::new(&req, Default::default());

    let endpoint = "foo".with("bar").with(path::<u32>()).join("baz".with(
        path_seq::<String>(),
    ));
    let f = match endpoint.apply(&req, &mut ctx) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("no route");
            return;
        }
    };

    let mut core = Core::new().unwrap();
    let result = core.run(f);
    println!("{:?}", result);
}
