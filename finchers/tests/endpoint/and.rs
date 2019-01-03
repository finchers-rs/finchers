use finchers::endpoint::{unit, value, EndpointExt};
use finchers::test;
use matches::assert_matches;

#[test]
fn test_and_all_ok() {
    let mut runner = test::runner(value("Hello").and(value("world")));

    assert_matches!(runner.apply_raw("/"), Ok(("Hello", "world")));
}

#[test]
fn test_and_flatten() {
    let mut runner = test::runner(
        value("Hello")
            .and(unit())
            .and(value("world").and(value(":)"))),
    );

    assert_matches!(runner.apply_raw("/"), Ok(("Hello", "world", ":)")));
}
