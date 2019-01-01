use finchers;
use finchers::prelude::*;

#[test]
#[ignore]
fn compiletest_apply_fn() {
    let endpoint = endpoint::apply_fn(|_cx| {
        Ok(finchers::future::poll_fn(|_| {
            Ok::<_, finchers::error::Never>(("foo", "bar").into())
        }))
    })
    .map(|x: &str, y: &str| format!("{}{}", x, y));

    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap();
}
