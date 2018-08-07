#![feature(rust_2018_preview)]

use finchers_core::ext::just;
use finchers_runtime::local::Client;

#[test]
fn test_just() {
    let endpoint = just("Alice");
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert_eq!(outcome, Some("Alice"));
}
