use finchers::endpoint::syntax;
use finchers::prelude::*;
use finchers::test;
use matches::assert_matches;

#[test]
fn test_or_1() {
    let mut runner = test::runner({
        let e1 = syntax::segment("foo").and(endpoint::value("foo"));
        let e2 = syntax::segment("bar").and(endpoint::value("bar"));
        e1.or(e2)
    });

    assert_matches!(runner.apply("/foo"), Ok(..));

    assert_matches!(runner.apply("/bar"), Ok(..));
}

#[test]
fn test_or_choose_longer_segments() {
    let mut runner = test::runner({
        let e1 = syntax::segment("foo") //
            .and(endpoint::value("foo"));
        let e2 = syntax::segment("foo")
            .and(syntax::segment("bar"))
            .and(endpoint::value("foobar"));
        e1.or(e2)
    });

    assert_matches!(runner.apply("/foo"), Ok(..));
    assert_matches!(runner.apply("/foo/bar"), Ok(..));
}
