extern crate finchers_endpoint;
extern crate finchers_test;

use finchers_endpoint::endpoint;
use finchers_endpoint::path::{path, path_req, paths};
use finchers_test::Client;

#[test]
fn test_endpoint_match_path() {
    let client = Client::new(endpoint("foo"));
    let outcome = client.get("/foo").run().unwrap();
    assert_eq!(outcome.ok(), Some(()));
}

#[test]
fn test_endpoint_reject_path() {
    let client = Client::new(endpoint("bar"));
    let outcome = client.get("/foo").run().unwrap();
    assert!(outcome.err().map_or(false, |e| e.is_noroute()));
}

#[test]
fn test_endpoint_match_multi_segments() {
    let client = Client::new(endpoint("/foo/bar"));
    let outcome = client.get("/foo/bar").run().unwrap();
    assert_eq!(outcome.ok(), Some(()));
}

#[test]
fn test_endpoint_reject_multi_segments() {
    let client = Client::new(endpoint("/foo/bar"));
    let outcome = client.get("/foo/baz").run().unwrap();
    assert!(outcome.err().map_or(false, |e| e.is_noroute()));
}

#[test]
fn test_endpoint_reject_short_path() {
    let client = Client::new(endpoint("/foo/bar/baz"));
    let outcome = client.get("/foo/bar").run().unwrap();
    assert!(outcome.err().map_or(false, |e| e.is_noroute()));
}

#[test]
fn test_endpoint_match_all_path() {
    let client = Client::new(endpoint("*"));
    let outcome = client.get("/foo").run().unwrap();
    assert_eq!(outcome.ok(), Some(()));
}

#[test]
fn test_endpoint_extract_integer() {
    let client = Client::new(path::<i32>());
    let outcome = client.get("/42").run().unwrap();
    assert_eq!(outcome.ok(), Some(42i32));
}

#[test]
fn test_endpoint_extract_wrong_integer() {
    let client = Client::new(path::<i32>());
    let outcome = client.get("/foo").run().unwrap();
    assert!(outcome.err().map_or(false, |e| e.is_noroute()));
}

#[test]
fn test_endpoint_extract_wrong_integer_result() {
    let client = Client::new(path::<Result<i32, _>>());
    let outcome = client.get("/foo").run().unwrap();
    match outcome.ok() {
        Some(Err(..)) => (),
        _ => panic!("assertion failed"),
    }
}

#[test]
fn test_endpoint_extract_wrong_integer_required() {
    let client = Client::new(path_req::<i32>());
    let outcome = client.get("/foo").run().unwrap();
    assert!(outcome.is_err());
}

#[test]
fn test_endpoint_extract_strings() {
    let client = Client::new(paths::<Vec<String>>());
    let outcome = client.get("/foo/bar").run().unwrap();
    assert_eq!(outcome.ok(), Some(vec!["foo".into(), "bar".into()]));
}
