#![feature(rust_2018_preview)]

use finchers_core::ext::{all, just};
use finchers_runtime::local::Client;

#[test]
fn test_and() {
    let endpoint = all(vec![just("Hello"), just("world")]);
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some(vec!["Hello", "world"]));
}
