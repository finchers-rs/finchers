#![feature(rust_2018_preview)]

use finchers_core::ext::lazy;
use finchers_runtime::local::Client;

#[test]
fn test_lazy_1() {
    let endpoint = lazy(|_| Some("Alice"));
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert_eq!(outcome, Some("Alice"));
}

#[test]
fn test_lazy_2() {
    let endpoint = lazy(|_| None as Option<()>);
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert!(outcome.is_none());
}
