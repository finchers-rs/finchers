use finchers::endpoint::EndpointExt;
use finchers::endpoints::path::{param, path, remains};
use finchers::local;
use http::StatusCode;

#[test]
fn test_match_single_segment() {
    let endpoint = path("foo");
    assert_matches!(local::get("/foo").apply(&endpoint), Ok(()));
    assert_matches!(
        local::get("/bar").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::NOT_FOUND
    );
}

#[test]
fn test_match_multi_segments() {
    let endpoint = path("foo").and(path("bar"));
    assert_matches!(local::get("/foo/bar").apply(&endpoint), Ok(()));
    assert_matches!(local::get("/foo/bar/").apply(&endpoint), Ok(()));
    assert_matches!(local::get("/foo/bar/baz").apply(&endpoint), Ok(()));
    assert_matches!(
        local::get("/foo").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::NOT_FOUND
    );
    assert_matches!(
        local::get("/foo/baz").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::NOT_FOUND
    );
}

#[test]
fn test_match_encoded_path() {
    let endpoint = path("foo/bar");
    assert_matches!(local::get("/foo%2Fbar").apply(&endpoint), Ok(()));
    assert_matches!(
        local::get("/foo/bar").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::NOT_FOUND
    );
}

#[test]
fn test_extract_integer() {
    let endpoint = param::<i32>();
    assert_matches!(local::get("/42").apply(&endpoint), Ok((42i32,)));
    assert_matches!(
        local::get("/foo").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::BAD_REQUEST
    );
}

#[test]
fn test_extract_strings() {
    let endpoint = path("foo").and(remains::<String>());
    assert_matches!(
        local::get("/foo/bar/baz/").apply(&endpoint),
        Ok((ref s,)) if s == "bar/baz/"
    );
}
