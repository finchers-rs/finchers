extern crate finchers_core;
extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_core::Never;
use finchers_core::error::NotPresent;
use finchers_endpoint::{abort, EndpointExt};
use finchers_test::Client;

#[test]
fn test_abort() {
    let client = Client::new(abort(|_| NotPresent::new("")).map(Never::never_into::<()>));

    let outcome = client.get("/").run();
    assert!(outcome.map_or(false, |r| r.is_err()));
}
