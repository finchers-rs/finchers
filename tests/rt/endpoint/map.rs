use finchers::prelude::*;
use finchers::rt::testing;

#[test]
fn test_map() {
    let mut runner = testing::runner(endpoint::cloned("Foo").map(|_| "Bar"));
    assert_matches!(runner.apply("/"), Ok("Bar"));
}
