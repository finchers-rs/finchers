#![feature(rust_2018_preview)]

use finchers_core::error::NotPresent;
use finchers_core::ext::{abort, EndpointExt};
use finchers_core::Never;
use finchers_runtime::local::Client;

#[test]
fn test_abort() {
    let client = Client::new(abort(|_| NotPresent::new("")).map(Never::never_into::<()>));

    let outcome = client.get("/").run();
    assert!(outcome.err().map_or(false, |e| !e.is_skipped()));
}
