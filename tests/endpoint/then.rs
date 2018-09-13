use finchers::local;
use finchers::prelude::*;

use futures_util::future::ready;
use matches::assert_matches;

#[test]
fn test_then() {
    let endpoint = endpoint::value("Foo").then(|_| ready("Bar"));

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Ok((ref s,)) if *s == "Bar"
    )
}
