#![feature(conservative_impl_trait)]

extern crate finch;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use finch::combinator::{path, path_seq};
use finch::either::Either;
use finch::endpoint::{Context, Endpoint, EndpointResult};
use finch::request::{Request, Body};

use hyper::{Method, Get};
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
struct Params(u32, Vec<String>);

fn endpoint1() -> impl Endpoint<Item = Params, Error = ()> + 'static {
    "foo"
        .with("bar")
        .with(path::<u32>())
        .skip("baz")
        .join(path_seq::<String>())
        .map(|(id, seq)| Params(id, seq))
}

fn endpoint2() -> impl Endpoint<Item = (), Error = ()> + 'static {
    "hello".with("world")
}

fn endpoint() -> impl Endpoint<Item = Either<Params, ()>, Error = ()> + 'static {
    endpoint1().or(endpoint2())
}

fn main() {
    println!(
        "{:?}",
        run_test(endpoint(), Get, "/foo/bar/42/baz/foo/bar/")
    );
    // => Ok(Ok(Either::A(Params(42, ["foo", "bar"]))))

    println!("{:?}", run_test(endpoint(), Get, "/hello/world"));
    // => Ok(Ok(Either::B(())))

    println!(
        "{:?}",
        run_test(endpoint(), Get, "/foo/baz/42/baz/foo/bar/")
    );
    // => Err(())
}
