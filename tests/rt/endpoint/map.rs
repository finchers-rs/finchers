use finchers::prelude::*;
use finchers::rt::test;

#[test]
fn test_map() {
    let mut runner = test::runner(endpoint::cloned("Foo").map(|_| "Bar"));
    assert_matches!(runner.apply("/"), Ok("Bar"));
}
