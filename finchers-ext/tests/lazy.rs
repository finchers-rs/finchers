extern crate finchers_ext;
extern crate finchers_test;

use finchers_ext::lazy;
use finchers_test::Client;

#[test]
fn test_lazy_1() {
    let endpoint = lazy(|_| Some("Alice"));
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some("Alice"));
}

#[test]
fn test_lazy_2() {
    let endpoint = lazy(|_| None as Option<()>);
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert!(outcome.err().map_or(false, |e| e.is_skipped()));
}
