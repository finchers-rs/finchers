use finchers::local;
use finchers::prelude::*;

use matches::assert_matches;

#[test]
fn test_map() {
    let endpoint = endpoint::value("Foo").map(|_| "Bar");

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Ok((ref s,)) if *s == "Bar"
    );
}
