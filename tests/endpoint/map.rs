use finchers::local;
use finchers::prelude::*;

#[test]
fn test_map() {
    let endpoint = endpoint::cloned("Foo").map(|_| "Bar");

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Ok((ref s,)) if *s == "Bar"
    );
}
