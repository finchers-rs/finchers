extern crate finchers_core;
extern crate finchers_ext;
extern crate finchers_test;

use finchers_core::error::NotPresent;
use finchers_core::Never;
use finchers_ext::{abort, EndpointExt};
use finchers_test::Client;

#[test]
fn test_abort() {
    let client = Client::new(abort(|_| NotPresent::new("")).map(Never::never_into::<()>));

    let outcome = client.get("/").run();
    assert!(outcome.err().map_or(false, |e| !e.is_skipped()));
}
