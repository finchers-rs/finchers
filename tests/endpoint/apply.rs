use finchers;
use finchers::prelude::*;

#[test]
#[ignore]
fn compiletest_apply() {
    let endpoint = endpoint::apply(|_cx| Ok(Ok("foo")));
    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap();
}

#[test]
#[ignore]
fn compiletest_apply_raw() {
    let endpoint = endpoint::apply_raw(|_cx| Ok(Ok(("foo", "bar"))))
        .map(|x: &str, y: &str| format!("{}{}", x, y));

    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap();
}
