extern crate finchers_ext;
extern crate finchers_test;

use finchers_ext::{just, EndpointResultExt};
use finchers_test::Client;

#[test]
fn test_as_ok() {
    let endpoint = just::<Result<_, ()>>(Ok("foo")).as_ok::<&str>();
    let client = Client::new(endpoint);

    let outcome: Result<Result<&str, ()>, _> = client.get("/").run();
    assert_eq!(outcome.ok(), Some(Ok("foo")));
}
