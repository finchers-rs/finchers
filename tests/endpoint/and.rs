use finchers::endpoint::{reject, unit, value, EndpointExt};
use finchers::error::NotPresent;
use finchers::rt::local;

#[test]
fn test_and_all_ok() {
    let endpoint = value("Hello").and(value("world"));

    assert_eq!(
        local::get("/").apply(&endpoint),
        Some(Ok(("Hello", "world"))),
    );
}

#[test]
fn test_and_with_err_1() {
    let endpoint = value("Hello").and(reject(|_| NotPresent::new("")).output::<()>());

    assert_eq!(
        local::get("/").apply(&endpoint).map(|res| res.is_err()),
        Some(true),
    );
}

#[test]
fn test_and_with_err_2() {
    let endpoint = reject(|_| NotPresent::new(""))
        .output::<()>()
        .and(value("Hello"));

    assert_eq!(
        local::get("/").apply(&endpoint).map(|res| res.is_err()),
        Some(true),
    );
}

#[test]
fn test_and_flatten() {
    let endpoint = value("Hello")
        .and(unit())
        .and(value("world").and(value(":)")));

    assert_eq!(
        local::get("/").apply(&endpoint),
        Some(Ok(("Hello", "world", ":)"))),
    );
}
