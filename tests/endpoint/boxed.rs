use finchers;
use finchers::endpoint::{EndpointObj, LocalEndpointObj};
use finchers::prelude::*;
use finchers::test;

#[test]
fn test_boxed() {
    let endpoint = path!(@get /"foo");
    let mut runner = test::runner(EndpointObj::new(endpoint));
    assert_matches!(runner.apply_raw("/foo"), Ok(()));
}

#[test]
fn test_boxed_local() {
    let endpoint = path!(@get /"foo");
    let mut runner = test::runner(LocalEndpointObj::new(endpoint));
    assert_matches!(runner.apply_raw("/foo"), Ok(..));
}

#[test]
fn smoke_test() {
    let endpoint = EndpointObj::new(path!(@get /"foo").map(|| "foo"));

    drop(move || {
        finchers::server::start(endpoint)
            .serve("127.0.0.1:4000")
            .expect("failed to start the server");
    });
}
