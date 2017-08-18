#![feature(conservative_impl_trait)]

extern crate finch;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use finch::combinator::{path, path_seq};
use finch::endpoint::{Context, Endpoint};
use finch::request::Request;

use hyper::{Method, Get};
use tokio_core::reactor::Core;

fn run_test<E: Endpoint>(
    endpoint: &E,
    method: Method,
    uri: &str,
) -> Result<Result<E::Item, E::Error>, ()> {
    let req = Request {
        method,
        uri: uri.parse().unwrap(),
        headers: Default::default(),
    };
    let mut ctx = Context::new(&req, Default::default());

    let f = endpoint.apply(&req, &mut ctx)?;

    let mut core = Core::new().unwrap();
    Ok(core.run(f))
}


fn main() {
    let endpoint = "foo".with("bar").with(path::<u32>()).join("baz".with(
        path_seq::<String>(),
    ));

    println!("{:?}", run_test(&endpoint, Get, "/foo/bar/42/baz/foo/bar/"));
    // => Ok(Ok((42, ["foo", "bar"])))


    println!("{:?}", run_test(&endpoint, Get, "/foo/baz/42/baz/foo/bar/"));
    // => Err(())
}
