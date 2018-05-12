extern crate finchers_core;
extern crate finchers_ext;
extern crate finchers_test;

use finchers_core::Never;
use finchers_core::error::NotPresent;
use finchers_ext::{abort, just, EndpointExt};
use finchers_test::Client;

#[test]
fn test_and_1() {
    let endpoint = just("Hello").and(just("world"));
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some(("Hello", "world")));
}

#[test]
fn test_and_2() {
    let endpoint = just("Hello").and(abort(|_| NotPresent::new("")).map(Never::never_into::<()>));
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert!(outcome.err().map_or(false, |e| !e.is_skipped()));
}
