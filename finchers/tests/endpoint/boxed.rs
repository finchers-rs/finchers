use finchers;
use finchers::endpoint::{syntax, EndpointObj, LocalEndpointObj};
use finchers::prelude::*;
use finchers::test;
use matches::assert_matches;

#[test]
fn test_boxed() {
    let endpoint = syntax::verb::get().and(syntax::segment("foo"));
    let mut runner = test::runner(EndpointObj::new(endpoint));
    assert_matches!(runner.apply_raw("/foo"), Ok(()));
}

#[test]
fn test_boxed_local() {
    let endpoint = syntax::verb::get().and(syntax::segment("foo"));
    let mut runner = test::runner(LocalEndpointObj::new(endpoint));
    assert_matches!(runner.apply_raw("/foo"), Ok(..));
}
