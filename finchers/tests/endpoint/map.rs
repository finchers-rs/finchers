use finchers::prelude::*;
use finchers::test;

#[test]
fn test_map() {
    let mut runner = test::runner(endpoint::value("Foo").map(|_| "Bar"));
    assert_matches!(runner.apply("/"), Ok("Bar"));
}
