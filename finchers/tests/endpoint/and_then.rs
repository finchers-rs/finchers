use finchers::error::{BadRequest, Error};
use finchers::prelude::*;
use finchers::test;
use futures::future;
use matches::assert_matches;

#[test]
fn test_and_then_1() {
    let mut runner = test::runner(endpoint::value("Foo").and_then(|_| future::ok("Bar")));
    assert_matches!(
        runner.apply("/"),
        Ok(s) if s == "Bar"
    )
}

#[test]
fn test_and_then_2() {
    let mut runner = test::runner(
        endpoint::value("Foo")
            .err_into::<Error>()
            .and_then(|_| future::err::<(), _>(BadRequest::from("Bar").into())),
    );
    assert_matches!(runner.apply("/"), Err(..))
}