#![feature(rust_2018_preview)]

use finchers_core::ext::{just, EndpointExt};
use finchers_runtime::local::Client;

#[test]
fn test_and_1() {
    let endpoint = just("Hello").and(just("world"));
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome, Some(("Hello", "world")));
}
