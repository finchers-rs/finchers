use finchers;
use finchers::prelude::*;

#[test]
fn smoketest_apply() {
    let endpoint = endpoint::apply(|_cx| Ok(Ok("foo")));

    drop(|| {
        finchers::server::start(endpoint)
            .serve_http("127.0.0.1:4000")
            .unwrap();
    })
}

#[test]
fn smoketest_apply_raw() {
    let endpoint = endpoint::apply_raw(|_cx| Ok(Ok(("foo", "bar"))))
        .map(|x: &str, y: &str| format!("{}{}", x, y));

    drop(|| {
        finchers::server::start(endpoint)
            .serve_http("127.0.0.1:4000")
            .unwrap();
    })
}
