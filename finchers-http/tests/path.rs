extern crate finchers_http;
extern crate finchers_test;

use finchers_http::path::{param, params, path};
use finchers_test::Client;

#[test]
fn test_endpoint_match_path() {
    let client = Client::new(path("foo"));
    let outcome = client.get("/foo").run();
    assert_eq!(outcome.and_then(Result::ok), Some(()));
}

#[test]
fn test_endpoint_reject_path() {
    let client = Client::new(path("bar"));
    let outcome = client.get("/foo").run();
    assert!(outcome.is_none());
}

#[test]
fn test_endpoint_match_multi_segments() {
    let client = Client::new(path("/foo/bar"));
    let outcome = client.get("/foo/bar").run();
    assert_eq!(outcome.and_then(Result::ok), Some(()));
}

#[test]
fn test_endpoint_reject_multi_segments() {
    let client = Client::new(path("/foo/bar"));
    let outcome = client.get("/foo/baz").run();
    assert!(outcome.is_none());
}

#[test]
fn test_endpoint_reject_short_path() {
    let client = Client::new(path("/foo/bar/baz"));
    let outcome = client.get("/foo/bar").run();
    assert!(outcome.is_none());
}

#[test]
fn test_endpoint_match_all_path() {
    let client = Client::new(path("*"));
    let outcome = client.get("/foo").run();
    assert_eq!(outcome.and_then(Result::ok), Some(()));
}

#[test]
fn test_endpoint_extract_integer() {
    let client = Client::new(param::<i32>());
    let outcome = client.get("/42").run();
    assert_eq!(outcome.and_then(Result::ok), Some(42i32));
}

#[test]
fn test_endpoint_extract_wrong_integer() {
    let client = Client::new(param::<i32>());
    let outcome = client.get("/foo").run();
    assert!(outcome.is_none());
}

#[test]
fn test_endpoint_extract_wrong_integer_result() {
    let client = Client::new(param::<Result<i32, _>>());
    let outcome = client.get("/foo").run();
    assert_eq!(outcome.map(|r| r.is_err()), Some(true));
}

#[test]
fn test_endpoint_extract_strings() {
    let client = Client::new(params::<Vec<String>>());
    let outcome = client.get("/foo/bar").run();
    assert_eq!(outcome.and_then(Result::ok), Some(vec!["foo".into(), "bar".into()]));
}
