extern crate finchers;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use finchers::Endpoint;
use finchers::combinator::method::get;
use finchers::combinator::path::{u32_, string_vec_, end_};
use finchers::response::Json;
use finchers::server::run_http;

#[derive(Debug, Serialize)]
enum Params {
    A(u32, Vec<String>),
    B,
}

fn main() {
    let new_endpoint = || {
        // "/foo/bar/<id:u32>/baz/<seq...:[String]>" => Params::A(id, seq)
        let e1 = get("foo".with("bar").with(u32_).skip("baz").join(string_vec_)).map(|(id, seq)| Params::A(id, seq));

        // "/hello/world" => Params::B
        let e2 = get("hello".with("world").skip(end_)).map(|()| Params::B);

        e1.or(e2).map(Json)
    };

    run_http(new_endpoint, "127.0.0.1:3000")
}
