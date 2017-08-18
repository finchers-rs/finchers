#![feature(conservative_impl_trait)]

extern crate finch;
extern crate futures;
extern crate hyper;

use finch::endpoint::Endpoint;
use finch::response::Responder;
use finch::server::EndpointService;
use finch::test::run_test;

use finch::combinator::get;
use finch::combinator::path::{u32_, string_vec_, end_};

use hyper::server::Http;


#[derive(Debug)]
enum Params {
    A(u32, Vec<String>),
    B,
}

impl From<(u32, Vec<String>)> for Params {
    fn from(val: (u32, Vec<String>)) -> Self {
        Params::A(val.0, val.1)
    }
}

impl Responder for Params {
    fn respond(self) -> hyper::Response {
        hyper::Response::new().with_body(format!("{:?}", self))
    }
}


#[cfg_attr(rustfmt, rustfmt_skip)]
fn endpoint() -> impl Endpoint<Item = Params, Error = &'static str> + 'static {
    // "/foo/bar/<id:u32>/baz/<seq...:[String]>" => Params::A(id, seq)
    let e1 = get("foo".with("bar").with(u32_).skip("baz").join(string_vec_))
        .map(Into::into)
        .map_err(|_| "endpoint 1");

    // "/hello/world" => Params::B
    let e2 = get("hello".with("world").skip(end_))
        .map(|_| Params::B)
        .map_err(|_| "endpoint 2");

    e1.or(e2)
}

fn main() {
    if false {
        do_test();
    }

    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new()
        .bind(&addr, || Ok(EndpointService(endpoint)))
        .unwrap();
    server.run().unwrap();
}


fn do_test() {
    use hyper::{Get, Post};

    println!("{:?}", run_test(endpoint, Get, "/foo/bar/42/baz/foo/bar/"));
    // => Ok(Ok(Params::A(42, ["foo", "bar"])))

    println!("{:?}", run_test(endpoint, Get, "/hello/world"));
    // => Ok(Ok(Params::B))

    println!("{:?}", run_test(endpoint, Get, "/foo/baz/42/baz/foo/bar/"));
    // => Err(RemainingPath)

    println!("{:?}", run_test(endpoint, Post, "/foo/bar/42/baz/foo/bar/"));
    // => Err(InvalidMethod)

    println!("{:?}", run_test(endpoint, Get, "/hello/world/hoge"));
    // => Err(NoRoute)
}
