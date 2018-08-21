use finchers::endpoint::{value, EndpointExt};
use finchers::local;

#[test]
fn test_map() {
    let endpoint = value("Foo").map(|_| "Bar");

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Ok((ref s,)) if *s == "Bar"
    );
}
