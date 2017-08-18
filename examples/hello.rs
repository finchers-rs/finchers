extern crate finchers;

use finchers::{Endpoint, Responder, Response};
use finchers::combinator::method::get;
use finchers::combinator::path::{u32_, string_vec_, end_};
use finchers::server::run_http;

#[derive(Debug)]
enum Params {
    A(u32, Vec<String>),
    B,
}

impl Responder for Params {
    fn respond(self) -> Response {
        Response::new().with_body(format!("{:?}", self))
    }
}


fn main() {
    let new_endpoint = || {
        // "/foo/bar/<id:u32>/baz/<seq...:[String]>" => Params::A(id, seq)
        let e1 = get("foo".with("bar").with(u32_).skip("baz").join(string_vec_))
            .map(|(id, seq)| Params::A(id, seq));

        // "/hello/world" => Params::B
        let e2 = get("hello".with("world").skip(end_)).map(|()| Params::B);

        e1.or(e2)
    };

    run_http(new_endpoint, "127.0.0.1:3000")
}
