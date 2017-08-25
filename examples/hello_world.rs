extern crate finchers;

use finchers::{Endpoint, Json};
use finchers::endpoint::{query, string_};
use finchers::endpoint::method::get;

fn main() {
    let endpoint = || {
        get("hello".with(string_))
            .join(query::<String>("foo"))
            .map(|(name, foo)| Json(format!("Hello, {}, {}", name, foo)))
    };

    finchers::server::run_http(endpoint, "127.0.0.1:3000");
}
