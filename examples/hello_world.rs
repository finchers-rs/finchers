#![allow(deprecated)]

extern crate finchers;

use finchers::Endpoint;
use finchers::endpoint::{param, query, segment};
use finchers::endpoint::method::get;
use finchers::json::Json;
use finchers::server::Server;

fn main() {
    let endpoint = |_: &_| {
        get(segment("hello").with(param()))
            .join(query("foo"))
            .map(|(name, foo): (String, String)| Json(format!("Hello, {}, {}", name, foo)))
            .with_type::<_, ()>()
    };

    Server::new(endpoint).bind("127.0.0.1:3000").run_http();
}
