use finchers::endpoint::{EndpointError, EndpointExt};
use finchers::endpoints::{method, path};
use finchers::local;
use futures_util::future::ready;
use http::Response;

#[test]
fn test_recover() {
    let endpoint = method::get(path::path("posts").and(path::param::<u32>()))
        .map(|id: u32| format!("param={}", id));

    let recovered = endpoint.fixed().recover(|err| {
        if err.is::<EndpointError>() {
            ready(Ok(Response::builder()
                .status(err.status_code())
                .body(err.status_code().to_string())
                .unwrap()))
        } else {
            ready(Err(err))
        }
    });

    assert!(local::get("/posts/42").apply(&recovered).is_ok());
    assert!(local::get("/posts/").apply(&recovered).is_ok());
    assert!(local::post("/posts/42").apply(&recovered).is_ok());
    assert!(local::get("/posts/foo").apply(&recovered).is_err());
}
