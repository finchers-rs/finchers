extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::{endpoint, ok, EndpointExt};
use finchers_test::Client;

#[test]
fn test_or_1() {
    let endpoint = endpoint("foo").right(ok("foo")).or(endpoint("bar").right(ok("bar")));
    let client = Client::new(endpoint);

    let outcome = client.get("/foo").run().unwrap();
    assert_eq!(outcome.ok(), Some("foo"));

    let outcome = client.get("/bar").run().unwrap();
    assert_eq!(outcome.ok(), Some("bar"));
}

#[test]
fn test_or_choose_longer_segments() {
    let e1 = endpoint("foo").right(ok("foo"));
    let e2 = endpoint("foo/bar").right(ok("foobar"));
    let endpoint = e1.or(e2);
    let client = Client::new(endpoint);

    let outcome = client.get("/foo").run().unwrap();
    assert_eq!(outcome.ok(), Some("foo"));

    let outcome = client.get("/foo/bar").run().unwrap();
    assert_eq!(outcome.ok(), Some("foobar"));
}
