use failure::format_err;
use finchers::endpoint::{reject, unit, value, EndpointExt};
use finchers::error::bad_request;
use finchers::local;

use http::StatusCode;
use matches::assert_matches;

#[test]
fn test_and_all_ok() {
    let endpoint = value("Hello").and(value("world"));

    assert_matches!(local::get("/").apply(&endpoint), Ok(("Hello", "world")));
}

#[test]
fn test_and_with_err_1() {
    let endpoint = value("Hello").and(reject(|_| bad_request(format_err!(""))).output::<()>());

    assert_matches!(
        local::get("/").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::BAD_REQUEST
    );
}

#[test]
fn test_and_with_err_2() {
    let endpoint = reject(|_| bad_request(format_err!("")))
        .output::<()>()
        .and(value("Hello"));

    assert_matches!(
        local::get("/").apply(&endpoint),
        Err(ref e) if e.status_code() == StatusCode::BAD_REQUEST
    );
}

#[test]
fn test_and_flatten() {
    let endpoint = value("Hello")
        .and(unit())
        .and(value("world").and(value(":)")));

    assert_matches!(
        local::get("/").apply(&endpoint),
        Ok(("Hello", "world", ":)"))
    );
}
