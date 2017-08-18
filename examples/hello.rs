#![feature(conservative_impl_trait)]

extern crate finch;
extern crate futures;
extern crate hyper;

use finch::endpoint::Endpoint;
use finch::response::Responder;
use finch::server::EndpointService;

use finch::combinator::get;
use finch::combinator::path::{u32_, string_vec_, end_};

use hyper::server::Http;


#[derive(Debug)]
enum Params {
    A(u32, Vec<String>),
    B,
}

impl Responder for Params {
    fn respond(self) -> hyper::Response {
        hyper::Response::new().with_body(format!("{:?}", self))
    }
}


fn main() {
    fn new_endpoint() -> impl Endpoint<Item = Params, Error = ()> + 'static {
        // "/foo/bar/<id:u32>/baz/<seq...:[String]>" => Params::A(id, seq)
        let e1 = get("foo".with("bar").with(u32_).skip("baz").join(string_vec_))
            .map(|(id, seq)| Params::A(id, seq));

        // "/hello/world" => Params::B
        let e2 = get("hello".with("world").skip(end_)).map(|()| Params::B);

        e1.or(e2)
    }

    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new()
        .bind(&addr, || Ok(EndpointService(new_endpoint)))
        .unwrap();
    server.run().unwrap();
}
