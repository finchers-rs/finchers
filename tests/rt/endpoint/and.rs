use finchers::endpoint::{cloned, unit, IntoEndpointExt};
use finchers::rt::testing;

#[test]
fn test_and_all_ok() {
    let mut runner = testing::runner(cloned("Hello").and(cloned("world")));

    assert_matches!(runner.apply_raw("/"), Ok(("Hello", "world")));
}

#[test]
fn test_and_flatten() {
    let mut runner = testing::runner(
        cloned("Hello")
            .and(unit())
            .and(cloned("world").and(cloned(":)"))),
    );

    assert_matches!(runner.apply_raw("/"), Ok(("Hello", "world", ":)")));
}
