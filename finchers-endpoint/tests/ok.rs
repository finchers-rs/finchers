extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::ok;
use finchers_test::Client;

#[test]
fn test_ok() {
    let endpoint = ok("Alice");
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some("Alice"));
}
