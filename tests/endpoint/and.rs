use finchers::endpoint::{cloned, unit, IntoEndpointExt};
use finchers::local;

#[test]
fn test_and_all_ok() {
    let endpoint = cloned("Hello").and(cloned("world"));

    assert_matches!(local::get("/").apply(&endpoint), Ok(("Hello", "world")));
}

#[test]
fn test_and_flatten() {
    let endpoint = cloned("Hello")
        .and(unit())
        .and(cloned("world").and(cloned(":)")));

    assert_matches!(
        local::get("/").apply(&endpoint),
        Ok(("Hello", "world", ":)"))
    );
}
