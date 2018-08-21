use finchers::endpoint::{value, EndpointExt};
use finchers::error::bad_request;
use finchers::rt::local;
use futures_util::future::ready;

#[test]
fn test_and_then_1() {
    let endpoint = value("Foo").and_then(|_| ready(Ok("Bar")));

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Ok((ref s,)) if *s == "Bar"
    )
}

#[test]
fn test_and_then_2() {
    let endpoint = value("Foo").and_then(|_| ready(Err::<(), _>(bad_request("Bar"))));

    assert_matches!(
        local::get("/")
            .apply(&endpoint),
        Err(ref e) if e.status_code().as_u16() == 400
    )
}
