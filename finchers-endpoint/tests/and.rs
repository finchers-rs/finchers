extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::{ok, EndpointExt};
use finchers_test::Client;

#[test]
fn test_and() {
    let endpoint = ok("Hello").and(ok("world"));
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some(("Hello", "world")));
}
