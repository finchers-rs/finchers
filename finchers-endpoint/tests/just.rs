extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::just;
use finchers_test::Client;

#[test]
fn test_just() {
    let endpoint = just("Alice");
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert_eq!(outcome.and_then(Result::ok), Some("Alice"));
}
