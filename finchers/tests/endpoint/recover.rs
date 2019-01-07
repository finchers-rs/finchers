use finchers::error::Error;
use finchers::prelude::*;
use finchers::test;
use http::StatusCode;
use matches::assert_matches;

#[test]
fn test_recover() {
    #[derive(Debug)]
    struct Id(Option<u32>);

    let mut runner = test::runner(
        endpoint::syntax::path!(@get "/foo/bar/<u32>")
            .map(|id| Id(Some(id)))
            .recover(|err: Error| {
                if err.is::<StatusCode>() {
                    Ok(Id(None))
                } else {
                    Err(err)
                }
            }),
    );

    assert_matches!(runner.apply("/foo/bar/42"), Ok(Id(Some(42))));
    assert_matches!(runner.apply("/foo/bar"), Ok(Id(None)));
    assert_matches!(runner.apply("/foo/bar/baz"), Err(..));
}
