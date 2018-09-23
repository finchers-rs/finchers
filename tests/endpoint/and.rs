use finchers::endpoint::{unit, value, IntoEndpointExt};
use finchers::local;

#[test]
fn test_and_all_ok() {
    let endpoint = value("Hello").and(value("world"));

    assert_matches!(local::get("/").apply(&endpoint), Ok(("Hello", "world")));
}

#[test]
fn test_and_flatten() {
    let endpoint = value("Hello")
        .and(unit())
        .and(value("world").and(value(":)")));

    assert_matches!(
        local::get("/").apply(&endpoint),
        Ok(("Hello", "world", ":)"))
    );
}
