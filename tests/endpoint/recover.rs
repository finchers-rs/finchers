use finchers::endpoint::syntax;
use finchers::endpoint::ApplyError;
use finchers::local;
use finchers::prelude::*;
use futures::future;
use http::Response;

#[test]
fn test_recover() {
    let endpoint = syntax::verb::get()
        .and(syntax::segment("posts"))
        .and(syntax::param::<u32>())
        .and(syntax::eos())
        .map(|id: u32| format!("param={}", id))
        .with_output::<(String,)>();

    let recovered = endpoint
        .wrap(endpoint::wrapper::or_reject())
        .wrap(endpoint::wrapper::recover(|err| {
            if err.is::<ApplyError>() {
                future::ok(
                    Response::builder()
                        .status(err.status_code())
                        .body(err.status_code().to_string())
                        .unwrap(),
                )
            } else {
                future::err(err)
            }
        }));

    assert!(local::get("/posts/42").apply(&recovered).is_ok());
    assert!(local::get("/posts/").apply(&recovered).is_ok());
    assert!(local::post("/posts/42").apply(&recovered).is_ok());
    assert!(local::get("/posts/foo").apply(&recovered).is_ok());
}
