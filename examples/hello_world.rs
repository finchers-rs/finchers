extern crate finchers;

use finchers::Endpoint;
use finchers::combinator::method::get;
use finchers::combinator::path::{string_, end_};
use finchers::response::Json;

fn main() {
    let new_endpoint = || {
        get("hello".with(string_).skip(end_).map(|name| {
            Json(format!("Hello, {}", name))
        }))
    };

    finchers::server::run_http(new_endpoint, "127.0.0.1:3000");
}
