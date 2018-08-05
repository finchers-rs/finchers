#![feature(rust_2018_preview)]

use finchers_core::ext::{just, EndpointResultExt};
use finchers_runtime::local::Client;

#[test]
fn test_as_ok() {
    let endpoint = just::<Result<_, ()>>(Ok("foo")).as_ok::<&str>();
    let client = Client::new(endpoint);

    let outcome: Result<Result<&str, ()>, _> = client.get("/").run();
    assert_eq!(outcome.ok(), Some(Ok("foo")));
}
