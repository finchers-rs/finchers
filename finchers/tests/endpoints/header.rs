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
    let mut runner = test::runner(
        endpoints::header::parse::<Mime>("content-type"), //
    );

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
fn test_header_optional() {
    let mut runner = test::runner(
        endpoints::header::optional::<Mime>("content-type"), //
    );

    assert_matches!(
        runner.apply(Request::post("/")
            .header("content-type", "application/json")),
        Ok(Some(ref m)) if *m == mime::APPLICATION_JSON
    );

    assert_matches!(runner.apply(Request::new(())), Ok(None));
}
