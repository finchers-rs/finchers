use finchers_core::endpoint::ext::{just, EndpointExt};
use finchers_core::future::ready;
use finchers_runtime::local::Client;

#[test]
fn test_then() {
    let endpoint = just(()).then(|| ready::<(Result<_, ()>,)>((Ok(()),)));
    let client = Client::new(endpoint);

    let output = client.get("/").run();
    assert!(output.is_some());
}
