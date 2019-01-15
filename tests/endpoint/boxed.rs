use finchers;
use finchers::endpoint::{syntax, IsEndpoint};
use finchers::prelude::*;
use finchers::test;
use matches::assert_matches;

#[test]
fn test_boxed() {
    let endpoint = syntax::verb::get().and(syntax::segment("foo"));
    let mut runner = test::runner(endpoint.boxed());
    assert_matches!(runner.apply_raw("/foo"), Ok(()));
}

#[test]
fn test_boxed_local() {
    let endpoint = syntax::verb::get().and(syntax::segment("foo"));
    let mut runner = test::runner(endpoint.boxed_local());
    assert_matches!(runner.apply_raw("/foo"), Ok(..));
}
