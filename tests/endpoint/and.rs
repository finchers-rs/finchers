use finchers_core::endpoint::{err, ok, EndpointExt};
use finchers_core::local;

#[test]
fn test_and_all_ok() {
    let endpoint = ok(("Hello",)).and(ok(("world",)));

    assert_eq!(
        local::get("/").apply(&endpoint),
        Some(Ok(("Hello", "world"))),
    );
}

#[test]
fn test_and_with_err_1() {
    let endpoint = ok(("Hello",)).and(err(()));

    assert_eq!(
        local::get("/").apply(&endpoint).map(|res| res.is_err()),
        Some(true),
    );
}

#[test]
fn test_and_with_err_2() {
    let endpoint = err(()).and(ok(("Hello",)));

    assert_eq!(
        local::get("/").apply(&endpoint).map(|res| res.is_err()),
        Some(true),
    );
}

#[test]
fn test_and_flatten() {
    let endpoint = ok(("Hello",)).and(ok(())).and(ok(("world", ":)")));

    assert_eq!(
        local::get("/").apply(&endpoint),
        Some(Ok(("Hello", "world", ":)"))),
    );
}
