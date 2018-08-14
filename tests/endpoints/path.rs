use finchers::endpoint::EndpointExt;
use finchers::endpoints::path::{param, path, remains};
use finchers::rt::local;

#[test]
fn test_match_single_segment() {
    let endpoint = path("foo");
    assert_eq!(local::get("/foo").apply(&endpoint), Some(Ok(())));
    assert_eq!(local::get("/bar").apply(&endpoint), None);
}

#[test]
fn test_match_multi_segments() {
    let endpoint = path("foo").and(path("bar"));
    assert_eq!(local::get("/foo/bar").apply(&endpoint), Some(Ok(())));
    assert_eq!(local::get("/foo/bar/").apply(&endpoint), Some(Ok(())));
    assert_eq!(local::get("/foo/bar/baz").apply(&endpoint), Some(Ok(())));
    assert_eq!(local::get("/foo").apply(&endpoint), None);
    assert_eq!(local::get("/foo/baz").apply(&endpoint), None);
}

#[test]
fn test_match_encoded_path() {
    let endpoint = path("foo/bar");
    assert_eq!(local::get("/foo%2Fbar").apply(&endpoint), Some(Ok(())));
    assert_eq!(local::get("/foo/bar").apply(&endpoint), None);
}

#[test]
fn test_extract_integer() {
    let endpoint = param::<i32>();
    assert_eq!(local::get("/42").apply(&endpoint), Some(Ok((42i32,))));
    assert_eq!(
        local::get("/foo").apply(&endpoint).map(|res| res.is_err()),
        Some(true)
    );
}

#[test]
fn test_extract_strings() {
    let endpoint = path("foo").and(remains::<String>());
    assert_eq!(
        local::get("/foo/bar/baz/").apply(&endpoint),
        Some(Ok(("bar/baz/".into(),)))
    );
}
