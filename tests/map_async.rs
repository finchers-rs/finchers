#![feature(rust_2018_preview)]

use finchers_core::ext::{just, EndpointExt};
use finchers_runtime::local::Client;

#[test]
fn test_map_async_2() {
    let endpoint = just(()).map_async(|_| Ok(()) as Result<_, ()>);
    let client = Client::new(endpoint);

    let output = client.get("/").run();
    assert!(output.is_ok());
}
