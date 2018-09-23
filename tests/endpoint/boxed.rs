use finchers;
use finchers::endpoint::{EndpointObj, LocalEndpointObj};
use finchers::local;
use finchers::prelude::*;

#[test]
fn test_boxed() {
    let endpoint = path!(@get /"foo");
    let endpoint = EndpointObj::new(endpoint);

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(()));
}

#[test]
fn test_boxed_local() {
    let endpoint = path!(@get /"foo");
    let endpoint = LocalEndpointObj::new(endpoint);

    assert_matches!(local::get("/foo").apply(&endpoint), Ok(..));
}

#[test]
fn smoke_test() {
    let endpoint = EndpointObj::new(path!(@get /"foo").map(|| "foo"));

    drop(move || {
        finchers::launch(endpoint).start("127.0.0.1:4000");
    });
}
