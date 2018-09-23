use finchers::error;
use finchers::local;
use finchers::prelude::*;

use http::header::CONTENT_TYPE;
use mime;
use mime::Mime;

#[test]
fn test_header_raw() {
    let endpoint = endpoints::header::raw(CONTENT_TYPE);

    assert_matches!(
        local::get("/")
            .header("content-type", "application/json")
            .apply(&endpoint),
        Ok((Some(ref value),)) if value.as_bytes() == &b"application/json"[..]
    );

    assert_matches!(local::get("/").apply(&endpoint), Ok((None,)));
}

#[test]
fn test_header_parse() {
    let endpoint = endpoints::header::parse::<Mime>("content-type").with_output::<(Mime,)>();

    assert_matches!(
        local::get("/")
            .header("content-type", "application/json")
            .apply(&endpoint),
        Ok((ref m,)) if *m == mime::APPLICATION_JSON
    );

    assert_matches!(
        local::get("/").apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    );
}

#[test]
fn test_header_parse_required() {
    let endpoint = endpoints::header::parse::<Mime>("content-type")
        .wrap(endpoint::wrapper::or_reject_with(|_, _| {
            error::bad_request("missing content-type")
        })).with_output::<(Mime,)>();

    assert_matches!(
        local::get("/")
            .header("content-type", "application/json")
            .apply(&endpoint),
        Ok((ref m,)) if *m == mime::APPLICATION_JSON
    );

    assert_matches!(
        local::get("/").apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
            && e.to_string() == "missing content-type"
    );
}

#[test]
fn test_header_optional() {
    let endpoint =
        endpoints::header::optional::<Mime>("content-type").with_output::<(Option<Mime>,)>();

    assert_matches!(
        local::get("/")
            .header("content-type", "application/json")
            .apply(&endpoint),
        Ok((Some(ref m),)) if *m == mime::APPLICATION_JSON
    );

    assert_matches!(local::get("/").apply(&endpoint), Ok((None,)));
}

#[test]
fn test_header_matches_with_rejection() {
    let endpoint = endpoints::header::matches("origin", "www.example.com")
        .wrap(endpoint::wrapper::or_reject_with(|_, _| {
            error::bad_request("The value of Origin is invalid")
        })).with_output::<()>();

    assert_matches!(
        local::get("/")
            .header("origin", "www.example.com")
            .apply(&endpoint),
        Ok(..)
    );

    // missing header
    assert_matches!(local::get("/").apply(&endpoint), Err(..));

    // invalid header value
    assert_matches!(
        local::get("/")
            .header("origin", "www.rust-lang.org")
            .apply(&endpoint),
        Err(..)
    );
}
