use finchers::endpoint::syntax;
use finchers::endpoint::{Endpoint, EndpointError, EndpointExt, IntoEndpointExt};
use finchers::local;
use finchers::path;

use http::StatusCode;
use matches::assert_matches;

#[test]
fn test_match_single_segment() {
    let endpoint = syntax::segment("foo");
    assert_matches!(local::get("/foo").apply(&endpoint), Ok(()));
    assert_matches!(
        local::get("/bar").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::NOT_FOUND
    );
}

#[test]
fn test_match_multi_segments() {
    let endpoint = syntax::segment("foo").and(syntax::segment("bar"));
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
    let endpoint = syntax::segment("foo/bar");
    assert_matches!(local::get("/foo%2Fbar").apply(&endpoint), Ok(()));
    assert_matches!(
        local::get("/foo/bar").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::NOT_FOUND
    );
}

#[test]
fn test_extract_integer() {
    let endpoint = syntax::param::<i32>();
    assert_matches!(local::get("/42").apply(&endpoint), Ok((42i32,)));
    assert_matches!(
        local::get("/foo").apply(&endpoint),
        Err(ref e) if e.is::<EndpointError>() && e.status_code() == StatusCode::BAD_REQUEST
    );
}

#[test]
fn test_extract_strings() {
    let endpoint = syntax::segment("foo").and(syntax::remains::<String>());
    assert_matches!(
        local::get("/foo/bar/baz/").apply(&endpoint),
        Ok((ref s,)) if s == "bar/baz/"
    );
}

#[test]
fn test_path_macro() {
    let endpoint = path!(@get / "posts" / u32 / "stars" /)
        .map(|id: u32| format!("id={}", id))
        .with_output::<(String,)>();
    assert_matches!(
        local::get("/posts/42/stars")
            .apply(&endpoint),
        Ok((ref s,)) if s == "id=42"
    );
}
