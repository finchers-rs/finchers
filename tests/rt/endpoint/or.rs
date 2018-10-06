use finchers::endpoint::syntax;
use finchers::error::bad_request;
use finchers::prelude::*;
use finchers::rt::test;

#[test]
fn test_or_1() {
    let mut runner = test::runner({
        let e1 = syntax::segment("foo").and(endpoint::cloned("foo"));
        let e2 = syntax::segment("bar").and(endpoint::cloned("bar"));
        e1.or(e2)
    });

    assert_matches!(runner.apply("/foo"), Ok(..));

    assert_matches!(runner.apply("/bar"), Ok(..));
}

#[test]
fn test_or_choose_longer_segments() {
    let mut runner = test::runner({
        let e1 = syntax::segment("foo").and(endpoint::cloned("foo"));
        let e2 = syntax::segment("foo")
            .and("bar")
            .and(endpoint::cloned("foobar"));
        e1.or(e2)
    });

    assert_matches!(runner.apply("/foo"), Ok(..));
    assert_matches!(runner.apply("/foo/bar"), Ok(..));
}

#[test]
fn test_or_with_rejection() {
    let mut runner = test::runner({
        syntax::segment("foo")
            .or(syntax::segment("bar"))
            .wrap(endpoint::wrapper::or_reject_with(|_err, _cx| {
                bad_request(format_err!("custom rejection"))
            }))
    });

    assert_matches!(runner.apply("/foo"), Ok(..));

    assert_matches!(
        runner.apply("/baz"),
        Err(ref e) if e.to_string() == "custom rejection"
    );
}
