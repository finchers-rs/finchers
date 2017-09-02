extern crate finchers;

use finchers::{Endpoint, Json};
use finchers::endpoint::{query, PathConst};
use finchers::endpoint::method::get;
use finchers::server::Server;

fn main() {
    let endpoint = |_: &_| {
        get("hello".with(String::PATH))
            .join(query::<String>("foo"))
            .map(|(name, foo)| Json(format!("Hello, {}, {}", name, foo)))
    };

    Server::new(endpoint).bind("127.0.0.1:3000").run_http();
}
