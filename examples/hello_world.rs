extern crate finchers;

use finchers::Endpoint;
use finchers::combinator::method::post;
use finchers::combinator::path::{string_, end_};
use finchers::combinator::param::param;
use finchers::combinator::body::take_body;
use finchers::response::Json;

fn main() {
    let new_endpoint = || {
        post("hello".with(string_).skip(end_).join3(
            param::<String>("foo"),
            take_body::<String>(),
        )).map(|(name, param, body)| {
            Json(vec![format!("Hello, {}, {}", name, param), body])
        })
    };

    finchers::server::run_http(new_endpoint, "127.0.0.1:3000");
}
