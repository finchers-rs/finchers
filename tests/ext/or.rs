use finchers_core::either::Either;
use finchers_core::endpoint::ext::{just, EndpointExt};
use finchers_core::http::path::path;
use finchers_runtime::local::Client;

#[test]
fn test_or_1() {
    let endpoint = (path("foo").and(just(("foo",)))).or(path("bar").and(just(("bar",))));
    let client = Client::new(endpoint);

    let outcome = client.get("/foo").run();
    assert_eq!(outcome, Some((Either::Left(("foo",)),)));

    let outcome = client.get("/bar").run();
    assert_eq!(outcome, Some((Either::Right(("bar",)),)));
}

#[test]
fn test_or_choose_longer_segments() {
    let e1 = path("foo").and(just(("foo",)));
    let e2 = path("foo/bar").and(just(("foobar",)));
    let endpoint = e1.or(e2);
    let client = Client::new(endpoint);

    let outcome = client.get("/foo").run();
    assert_eq!(outcome, Some((Either::Left(("foo",)),)));

    let outcome = client.get("/foo/bar").run();
    assert_eq!(outcome, Some((Either::Right(("foobar",)),)));
}
