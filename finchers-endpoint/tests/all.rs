extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::{all, just};
use finchers_test::Client;

#[test]
fn test_and() {
    let endpoint = all(vec![just("Hello"), just("world")]);
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.and_then(Result::ok), Some(vec!["Hello", "world"]));
}
