use failure::format_err;
use finchers::endpoint::{reject, value, EndpointExt};
use finchers::endpoints::header;
use finchers::endpoints::path::path;
use finchers::error::bad_request;
use finchers::rt::local;

#[test]
fn test_or_1() {
    let e1 = path("foo").and(value("foo"));
    let e2 = path("bar").and(value("bar"));
    let endpoint = e1.or(e2);

    assert_eq!(local::get("/foo").apply(&endpoint), Some(Ok(("foo",))),);

    assert_eq!(local::get("/bar").apply(&endpoint), Some(Ok(("bar",))),);
}

#[test]
fn test_or_choose_longer_segments() {
    let e1 = path("foo").and(value("foo"));
    let e2 = path("foo").and(path("bar")).and(value("foobar"));
    let endpoint = e1.or(e2);

    assert_eq!(local::get("/foo").apply(&endpoint), Some(Ok(("foo",))),);

    assert_eq!(
        local::get("/foo/bar").apply(&endpoint),
        Some(Ok(("foobar",))),
    );
}

#[test]
fn test_or_with_rejection_path() {
    let endpoint = path("foo")
        .or(path("bar"))
        .or(reject(|_| bad_request(format_err!("custom rejection"))));

    assert_eq!(
        local::get("/foo")
            .apply(&endpoint)
            .map(|res| res.map_err(|e| e.to_string())),
        Some(Ok(())),
    );

    assert_eq!(
        local::get("/baz")
            .apply(&endpoint)
            .map(|res| res.map_err(|e| e.to_string())),
        Some(Err("custom rejection".into()))
    );
}

#[test]
fn test_or_with_rejection_header() {
    use finchers::input::header::FromHeader;
    use http::header::HeaderValue;
    use std::str;

    #[derive(Debug, PartialEq)]
    struct Authorization(String);

    impl FromHeader for Authorization {
        const HEADER_NAME: &'static str = "authorization";
        type Error = str::Utf8Error;
        fn from_header(v: &HeaderValue) -> Result<Self, Self::Error> {
            str::from_utf8(v.as_bytes())
                .map(ToOwned::to_owned)
                .map(Authorization)
        }
    }

    let endpoint = header::optional::<Authorization>().or(reject(|_| {
        bad_request(format_err!("missing authorization header"))
    }));

    assert_eq!(
        local::get("/")
            .header("authorization", "Basic xxxx")
            .apply(&endpoint)
            .map(|res| res.map_err(|e| e.to_string())),
        Some(Ok((Authorization("Basic xxxx".into()),))),
    );

    assert_eq!(
        local::get("/")
            .apply(&endpoint)
            .map(|res| res.map_err(|e| e.to_string())),
        Some(Err("missing authorization header".into()))
    );
}
