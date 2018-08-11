use finchers_core::endpoint::ext::result::ok;
use finchers_core::endpoint::ext::EndpointResultExt;
use finchers_runtime::local::Client;

#[test]
fn test_as_ok() {
    let endpoint = ok::<_, ()>("foo").as_ok::<&str>();
    let client = Client::new(endpoint);

    let outcome = client.get("/").run();
    assert_eq!(outcome, Some((Ok("foo"),)));
}
