use finchers::error::bad_request;
use finchers::local;
use finchers::prelude::*;

use futures::future;

#[test]
fn test_and_then_1() {
    let endpoint = endpoint::value("Foo").and_then(|_| future::ok("Bar"));

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Ok((ref s,)) if *s == "Bar"
    )
}

#[test]
fn test_and_then_2() {
    let endpoint = endpoint::value("Foo").and_then(|_| future::err::<(), _>(bad_request("Bar")));

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    )
}
