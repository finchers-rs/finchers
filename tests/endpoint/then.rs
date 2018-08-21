use finchers::endpoint::{value, EndpointExt};
use finchers::rt::local;
use futures_util::future::ready;

#[test]
fn test_then() {
    let endpoint = value("Foo").then(|_| ready("Bar"));

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Ok((ref s,)) if *s == "Bar"
    )
}
