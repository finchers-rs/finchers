use finchers::endpoint::{value, EndpointExt};
use finchers::local;
use futures_util::future::poll_fn;

#[test]
fn test_and_then_with() {
    let prefix = String::from("Hello, ");
    let endpoint =
        value("Alice").and_then_with(prefix, |prefix: &String, (name,): (&'static str,)| {
            poll_fn(move |_cx| {
                // This closure captures a `String` by reference.
                // Therefore, the Future returned by `poll_fn()` inherits the lifetime
                // of `prefix`.
                Ok(format!("{}{}.", prefix, name)).into()
            })
        });

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Ok((ref s,)) if *s == "Hello, Alice."
    )
}
