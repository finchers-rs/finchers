use finchers::error;
use finchers::prelude::*;
use finchers::test;

use http::header::CONTENT_TYPE;
use http::Request;
use matches::assert_matches;
use mime;
use mime::Mime;

#[test]
fn test_header_raw() {
    let mut runner = test::runner(endpoints::header::raw(CONTENT_TYPE));

    assert_matches!(
        runner.apply(Request::get("/")
            .header("content-type", "application/json")),
        Ok(Some(ref value)) if value.as_bytes() == &b"application/json"[..]
    );

    assert_matches!(runner.apply(Request::new(())), Ok(None));
}

#[test]
fn test_header_parse() {
    let mut runner =
        test::runner({ endpoints::header::parse::<Mime>("content-type").with_output::<(Mime,)>() });

    assert_matches!(
        runner.apply(Request::post("/")
            .header("content-type", "application/json")),
        Ok(ref m) if *m == mime::APPLICATION_JSON
    );

    assert_matches!(
        runner.apply(Request::new(())),
        Err(ref e) if e.status_code().as_u16() == 400
    );
}

#[test]
fn test_header_parse_required() {
    let mut runner = test::runner({
        endpoints::header::parse::<Mime>("content-type")
            .wrap(endpoint::wrapper::or_reject_with(|_, _| {
                error::bad_request("missing content-type")
            }))
            .with_output::<(Mime,)>()
    });

    assert_matches!(
        runner.apply(Request::post("/")
            .header("content-type", "application/json")),
        Ok(ref m) if *m == mime::APPLICATION_JSON
    );

    assert_matches!(
        runner.apply(Request::new(())),
        Err(ref e) if e.status_code().as_u16() == 400
            && e.to_string() == "missing content-type"
    );
}

#[test]
fn test_header_optional() {
    let mut runner = test::runner({
        endpoints::header::optional::<Mime>("content-type").with_output::<(Option<Mime>,)>()
    });

    assert_matches!(
        runner.apply(Request::post("/")
            .header("content-type", "application/json")),
        Ok(Some(ref m)) if *m == mime::APPLICATION_JSON
    );

    assert_matches!(runner.apply(Request::new(())), Ok(None));
}

#[test]
fn test_header_matches_with_rejection() {
    let mut runner = test::runner({
        endpoints::header::matches("origin", "www.example.com")
            .wrap(endpoint::wrapper::or_reject_with(|_, _| {
                error::bad_request("The value of Origin is invalid")
            }))
            .with_output::<()>()
    });

    assert_matches!(
        runner.apply_raw(Request::get("/").header("origin", "www.example.com")),
        Ok(..)
    );

    // missing header
    assert_matches!(runner.apply_raw(Request::new(())), Err(..));

    // invalid header value
    assert_matches!(
        runner.apply_raw(Request::get("/").header("origin", "www.rust-lang.org")),
        Err(..)
    );
}
