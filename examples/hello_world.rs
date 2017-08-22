extern crate finchers;

use finchers::{Endpoint, Json};
use finchers::endpoint::{param, string_};
use finchers::endpoint::method::get;

fn main() {
    let endpoint = get("hello".with(string_))
        .join(param::<String>("foo"))
        .map(|(name, param)| Json(format!("Hello, {}, {}", name, param)));

    finchers::server::run_http(endpoint, "127.0.0.1:3000");
}
