extern crate finchers_ext;
extern crate finchers_test;

use finchers_ext::just;
use finchers_test::Client;

#[test]
fn test_just() {
    let endpoint = just("Alice");
    let client = Client::new(endpoint);
    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some("Alice"));
}
