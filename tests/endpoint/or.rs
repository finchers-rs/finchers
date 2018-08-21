use failure::format_err;
use finchers::endpoint::{reject, value, EndpointExt};
use finchers::endpoints::path::path;
use finchers::error::bad_request;
use finchers::rt::local;

#[test]
fn test_or_1() {
    let e1 = path("foo").and(value("foo"));
    let e2 = path("bar").and(value("bar"));
    let endpoint = e1.or(e2);

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(..));

    assert_matches!(local::get("/bar").apply(&endpoint), Ok(..));
}

#[test]
fn test_or_choose_longer_segments() {
    let e1 = path("foo").and(value("foo"));
    let e2 = path("foo").and(path("bar")).and(value("foobar"));
    let endpoint = e1.or(e2);

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(..));

    assert_matches!(local::get("/foo/bar").apply(&endpoint), Ok(..));
}

#[test]
fn test_or_with_rejection_path() {
    let endpoint = path("foo")
        .or(path("bar"))
        .or(reject(|_| bad_request(format_err!("custom rejection"))));

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(..));

    assert_matches!(
        local::get("/baz").apply(&endpoint),
        Err(ref e) if e.to_string() == "custom rejection"
    );
}
