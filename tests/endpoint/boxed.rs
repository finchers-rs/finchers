use finchers::endpoint::EndpointExt;
use finchers::local;
use finchers::route;

use matches::assert_matches;

#[test]
fn test_boxed() {
    let endpoint = route!(@get /"foo");
    let endpoint = endpoint.boxed::<()>();

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(()));
}

#[test]
fn test_boxed_local() {
    let endpoint = route!(@get /"foo");
    let endpoint = endpoint.boxed_local::<()>();

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(..));
}
