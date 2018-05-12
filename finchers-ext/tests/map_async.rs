extern crate finchers_core;
extern crate finchers_ext;
extern crate finchers_test;

use finchers_ext::{just, EndpointExt};
use finchers_test::Client;

#[test]
fn test_map_async_2() {
    let endpoint = just(()).map_async(|_| Ok(()) as Result<_, ()>);
    let client = Client::new(endpoint);

    let output = client.get("/").run();
    assert!(output.is_ok());
}
