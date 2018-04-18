extern crate finchers_core;
extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_core::error::NotPresent;
use finchers_endpoint::{abort, ok, EndpointExt};
use finchers_test::Client;

#[test]
fn test_and_1() {
    let endpoint = ok("Hello").and(ok("world"));
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some(("Hello", "world")));
}

#[test]
fn test_and_2() {
    let endpoint = ok("Hello").and(abort(|_| NotPresent::new("")).as_::<!>());
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert!(outcome.err().map_or(false, |e| e.is_aborted()));
}
