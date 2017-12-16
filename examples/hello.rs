extern crate finchers;

use finchers::Endpoint;
use finchers::endpoint::method::get;
use finchers::endpoint::param;
use finchers::ServerBuilder;

fn main() {
    // GET /foo/:id/bar
    let endpoint = get(("foo", param(), "bar"))
        .map(|(_, name, _)| name)
        .and_then(|name: String| -> Result<_, ()> { Ok(format!("Hello, {}", name)) });

    ServerBuilder::default()
        .bind("0.0.0.0:8080")
        .num_workers(1)
        .run_http(endpoint);
}
