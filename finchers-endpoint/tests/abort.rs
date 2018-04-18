extern crate finchers_core;
extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_core::error::NotPresent;
use finchers_endpoint::{abort, EndpointExt};
use finchers_test::Client;

#[test]
fn test_abort() {
    let client = Client::new(abort(|_| NotPresent::new("")).as_::<!>());

    let outcome = client.get("/").run();
    assert!(outcome.err().map_or(false, |e| e.is_aborted()));
}
