extern crate finchers_ext;
extern crate finchers_test;

use finchers_ext::{all, just};
use finchers_test::Client;

#[test]
fn test_and() {
    let endpoint = all(vec![just("Hello"), just("world")]);
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome.ok(), Some(vec!["Hello", "world"]));
}
