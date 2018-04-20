extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::lazy;
use finchers_test::Client;

#[test]
fn test_lazy_1() {
    let endpoint = lazy(|_| Some("Alice"));
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert_eq!(outcome.and_then(Result::ok), Some("Alice"));
}

#[test]
fn test_lazy_2() {
    let endpoint = lazy(|_| None as Option<()>);
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert!(outcome.is_none());
}
