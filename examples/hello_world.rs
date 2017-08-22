extern crate finchers;

use finchers::{Endpoint, Json};
use finchers::combinator::method::get;
use finchers::combinator::path::{end_, string_};
use finchers::combinator::param::param;

fn main() {
    let endpoint = get(
        "hello"
            .with(string_)
            .skip(end_)
            .join(param::<String>("foo")),
    ).map(|(name, param)| Json(format!("Hello, {}, {}", name, param)));

    finchers::server::run_http(endpoint, "127.0.0.1:3000");
}
