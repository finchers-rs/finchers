extern crate finchers_core;
extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_core::error::NotPresent;
use finchers_endpoint::{ok, EndpointExt};
use finchers_test::Client;

#[test]
fn test_and_then_1() {
    let endpoint = ok(()).and_then(|_| Err(NotPresent::new("an error")) as Result<(), _>);
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert!(outcome.err().map_or(false, |e| e.is_aborted()));
}

#[test]
fn test_and_then_2() {
    let endpoint = ok(()).and_then(|_| Ok(()) as Result<_, NotPresent>);
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert!(outcome.is_ok());
}
