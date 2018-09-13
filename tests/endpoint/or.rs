use failure::format_err;
use finchers::endpoint::syntax;
use finchers::endpoint::{value, EndpointExt, IntoEndpointExt};
use finchers::error::bad_request;
use finchers::local;
use matches::assert_matches;

#[test]
fn test_or_1() {
    let e1 = syntax::segment("foo").and(value("foo"));
    let e2 = syntax::segment("bar").and(value("bar"));
    let endpoint = e1.or(e2);

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(..));

    assert_matches!(local::get("/bar").apply(&endpoint), Ok(..));
}

#[test]
fn test_or_choose_longer_segments() {
    let e1 = syntax::segment("foo").and(value("foo"));
    let e2 = syntax::segment("foo").and("bar").and(value("foobar"));
    let endpoint = e1.or(e2);

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(..));

    assert_matches!(local::get("/foo/bar").apply(&endpoint), Ok(..));
}

#[test]
fn test_or_with_rejection() {
    let endpoint = syntax::segment("foo")
        .or(syntax::segment("bar"))
        .or_reject_with(|_err, _cx| bad_request(format_err!("custom rejection")));

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(..));

    assert_matches!(
        local::get("/baz").apply(&endpoint),
        Err(ref e) if e.to_string() == "custom rejection"
    );
}
