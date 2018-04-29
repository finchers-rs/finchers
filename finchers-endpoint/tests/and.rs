extern crate finchers_core;
extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_core::Never;
use finchers_core::error::NotPresent;
use finchers_endpoint::{abort, just, EndpointExt};
use finchers_test::Client;

#[test]
fn test_and_1() {
    let endpoint = just("Hello").and(just("world"));
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.and_then(Result::ok), Some(("Hello", "world")));
}

#[test]
fn test_and_2() {
    let endpoint = just("Hello").and(abort(|_| NotPresent::new("")).map(Never::never_into::<()>));
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert!(outcome.map_or(false, |r| r.is_err()));
}
