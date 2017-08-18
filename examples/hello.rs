#![feature(conservative_impl_trait)]

extern crate finch;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use finch::endpoint::{Context, Endpoint};
use finch::errors::EndpointResult;
use finch::request::{Request, Body};

use finch::combinator::get;
use finch::combinator::path::{u32_, string_vec_, end_};

use hyper::{Method, Get, Post};
use tokio_core::reactor::Core;

fn run_test<E: Endpoint>(
    endpoint: E,
    method: Method,
    uri: &str,
) -> EndpointResult<Result<E::Item, E::Error>>
where
    E::Item: std::fmt::Debug,
    E::Error: std::fmt::Debug,
{
    let req = Request {
        method,
        uri: uri.parse().unwrap(),
        headers: Default::default(),
        body: Some(Body::default()),
    };
    let ctx = Context::new(&req);

    let (_ctx, f) = endpoint.apply(ctx)?;

    let mut core = Core::new().unwrap();
    Ok(core.run(f))
}


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
    println!(
        "{:?}",
        run_test(endpoint(), Get, "/foo/bar/42/baz/foo/bar/")
    );
    // => Ok(Ok(Params::A(42, ["foo", "bar"])))

    println!("{:?}", run_test(endpoint(), Get, "/hello/world"));
    // => Ok(Ok(Params::B))

    println!(
        "{:?}",
        run_test(endpoint(), Get, "/foo/baz/42/baz/foo/bar/")
    );
    // => Err(NoRoute)

    println!(
        "{:?}",
        run_test(endpoint(), Post, "/foo/bar/42/baz/foo/bar/")
    );
    // => Err(InvalidMethod)

    println!("{:?}", run_test(endpoint(), Get, "/hello/world/hoge"));
    // => Err(NoRoute)
}
