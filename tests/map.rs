#![feature(rust_2018_preview)]

use finchers_core::ext::{just, EndpointExt};
use finchers_runtime::local::Client;

#[test]
fn test_map() {
    let endpoint = just(()).map(|_| "Foo");
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some("Foo"));
}
