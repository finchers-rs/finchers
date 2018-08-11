use finchers_core::endpoint::{ok, EndpointExt};
use finchers_core::endpoints::path::path;
use finchers_core::local;

#[test]
fn test_or_1() {
    let e1 = path("foo").and(ok(("foo",)));
    let e2 = path("bar").and(ok(("bar",)));
    let endpoint = e1.or(e2);

    assert_eq!(local::get("/foo").apply(&endpoint), Some(Ok(("foo",))),);

    assert_eq!(local::get("/bar").apply(&endpoint), Some(Ok(("bar",))),);
}

#[test]
fn test_or_choose_longer_segments() {
    let e1 = path("foo").and(ok(("foo",)));
    let e2 = path("foo/bar").and(ok(("foobar",)));
    let endpoint = e1.or(e2);

    assert_eq!(local::get("/foo").apply(&endpoint), Some(Ok(("foo",))),);

    assert_eq!(
        local::get("/foo/bar").apply(&endpoint),
        Some(Ok(("foobar",))),
    );
}
