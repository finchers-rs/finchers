extern crate finchers;

use finchers::{Endpoint, Json};
use finchers::combinator::method::post;
use finchers::combinator::path::{string_, end_};
use finchers::combinator::param::param;
use finchers::combinator::body::body;

fn main() {
    let new_endpoint = || {
        post(
            "hello"
                .with(string_)
                .skip(end_)
                .join(param::<String>("foo"))
                .join(body::<Json>()),
        ).map(|((name, param), body)| {
            Json(vec![
                format!("Hello, {}, {}", name, param),
                format!("{:?}", body),
            ])
        })
    };

    finchers::server::run_http(new_endpoint, "127.0.0.1:3000");
}
