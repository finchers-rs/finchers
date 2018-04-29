extern crate finchers_core;
extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_core::error::NotPresent;
use finchers_endpoint::{just, EndpointExt};
use finchers_test::Client;

#[test]
fn test_then_1() {
    let endpoint = just(()).then(|_| Err(NotPresent::new("an error")) as Result<(), _>);
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert!(outcome.map_or(false, |r| r.ok().map_or(false, |output| output.is_err())));
}

#[test]
fn test_then_2() {
    let endpoint = just(()).then(|_| Ok(()) as Result<_, NotPresent>);
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.map(|r| r.is_ok()), Some(true));
}
