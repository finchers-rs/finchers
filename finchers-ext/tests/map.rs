extern crate finchers_core;
extern crate finchers_ext;
extern crate finchers_test;

use finchers_ext::{just, EndpointExt};
use finchers_test::Client;

#[test]
fn test_map() {
    let endpoint = just(()).map(|_| "Foo");
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.and_then(Result::ok), Some("Foo"));
}
