#![feature(rust_2018_preview)]

use finchers_core::http::path::{param, params, path};
use finchers_runtime::local::Client;

#[test]
fn test_match_path() {
    let client = Client::new(path("foo"));
    let outcome = client.get("/foo").run();
    assert_eq!(outcome, Some(()));
}

#[test]
fn test_reject_path() {
    let client = Client::new(path("bar"));
    let outcome = client.get("/foo").run();
    assert!(outcome.is_none());
}

#[test]
fn test_match_multi_segments() {
    let client = Client::new(path("/foo/bar"));
    let outcome = client.get("/foo/bar").run();
    assert_eq!(outcome, Some(()));
}

#[test]
fn test_reject_multi_segments() {
    let client = Client::new(path("/foo/bar"));
    let outcome = client.get("/foo/baz").run();
    assert!(outcome.is_none());
}

#[test]
fn test_reject_short_path() {
    let client = Client::new(path("/foo/bar/baz"));
    let outcome = client.get("/foo/bar").run();
    assert!(outcome.is_none());
}

#[test]
fn test_match_all_path() {
    let client = Client::new(path("*"));
    let outcome = client.get("/foo").run();
    assert_eq!(outcome, Some(()));
}

#[test]
fn test_match_encoded_path() {
    let client = Client::new(path("foo%2Fbar"));
    let outcome = client.get("/foo%2Fbar").run();
    assert_eq!(outcome, Some(()));
}

#[test]
fn test_reject_encoded_path() {
    let client = Client::new(path("foo%2Fbar"));
    let outcome = client.get("/foo/bar").run();
    assert!(outcome.is_none());
}

#[test]
fn test_extract_integer() {
    let client = Client::new(param::<i32>());
    let outcome = client.get("/42").run();
    assert_eq!(outcome.and_then(Result::ok), Some(42i32));
}

#[test]
fn test_extract_wrong_integer() {
    let client = Client::new(param::<i32>());
    let outcome = client.get("/foo").run();
    assert!(outcome.map_or(false, |res| res.is_err()));
}

#[test]
fn test_extract_strings() {
    let client = Client::new(params::<Vec<String>>());
    let outcome = client.get("/foo/bar").run();
    assert_eq!(outcome, Some(vec!["foo".into(), "bar".into()]));
}
