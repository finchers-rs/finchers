extern crate finchers_core;
extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::{ok, EndpointExt};
use finchers_test::Client;

#[test]
fn test_map() {
    let endpoint = ok(()).map(|_| "Foo");
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some("Foo"));
}
