#![allow(deprecated)]

extern crate finchers;

use finchers::Endpoint;
use finchers::endpoint::{param, query, segment};
use finchers::endpoint::method::get;
use finchers::json::Json;
use finchers::server::Server;

fn main() {
    let endpoint = |_: &_| {
        get(segment("hello").with(param::<String, ()>()))
            .join(query::<String, ()>("foo"))
            .map(|(name, foo)| Json(format!("Hello, {}, {}", name, foo)))
    };

    Server::new(endpoint).bind("127.0.0.1:3000").run_http();
}
