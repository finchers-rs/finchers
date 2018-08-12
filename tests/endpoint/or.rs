use finchers_core::endpoint::{ok, reject, EndpointExt};
use finchers_core::endpoints::header;
use finchers_core::endpoints::path::path;
use finchers_core::local;

#[test]
fn test_or_1() {
    let e1 = path("foo").and(ok(("foo",)));
    let e2 = path("bar").and(ok(("bar",)));
    let endpoint = e1.or(e2);

    assert_eq!(local::get("/foo").apply(&endpoint), Some(Ok(("foo",))),);

    assert_eq!(local::get("/bar").apply(&endpoint), Some(Ok(("bar",))),);
}

#[test]
fn test_or_choose_longer_segments() {
    let e1 = path("foo").and(ok(("foo",)));
    let e2 = path("foo/bar").and(ok(("foobar",)));
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
        .or(reject(|_| "custom rejection"));

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
    let endpoint =
        header::parse::<String>("authorization").or(reject(|_| "missing authorization header"));

    assert_eq!(
        local::get("/")
            .header("authorization", "Basic xxxx")
            .apply(&endpoint)
            .map(|res| res.map_err(|e| e.to_string())),
        Some(Ok(("Basic xxxx".into(),))),
    );

    assert_eq!(
        local::get("/")
            .apply(&endpoint)
            .map(|res| res.map_err(|e| e.to_string())),
        Some(Err("missing authorization header".into()))
    );
}
