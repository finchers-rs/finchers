extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::{just, EndpointResultExt};
use finchers_test::Client;

#[test]
fn test_as_ok() {
    let endpoint = just::<Result<_, ()>>(Ok("foo")).as_ok::<&str>();
    let client = Client::new(endpoint);

    let outcome: Option<Result<&str, ()>> = client.get("/").run().and_then(Result::ok);
    assert_eq!(outcome, Some(Ok("foo")));
}
