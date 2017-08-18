#![feature(conservative_impl_trait)]

extern crate finch;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use finch::combinator::{path, path_seq, get};
use finch::either::Either;
use finch::endpoint::{Context, Endpoint, EndpointResult};
use finch::request::{Request, Body};

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

fn endpoint() -> impl Endpoint<Item = Params, Error = ()> + 'static {
    let e1 = "foo".with("bar").with(path::<u32>()).skip("baz").join(
        path_seq::<String>(),
    );

    let e2 = "hello".with("world");

    get(e1).or(get(e2)).map(|e| match e {
        Either::A((id, seq)) => Params::A(id, seq),
        Either::B(()) => Params::B,
    })
}

fn main() {
    println!(
        "{:?}",
        run_test(endpoint(), Get, "/foo/bar/42/baz/foo/bar/")
    );
    // => Ok(Ok(Either::A(Params(42, ["foo", "bar"]))))

    println!(
        "{:?}",
        run_test(endpoint(), Post, "/foo/bar/42/baz/foo/bar/")
    );
    // => Err(InvalidMethod)

    println!("{:?}", run_test(endpoint(), Get, "/hello/world"));
    // => Ok(Ok(Either::B(())))

    println!(
        "{:?}",
        run_test(endpoint(), Get, "/foo/baz/42/baz/foo/bar/")
    );
    // => Err(NoRoute)
}
