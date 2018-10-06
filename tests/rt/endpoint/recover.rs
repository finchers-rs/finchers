use finchers::endpoint::syntax;
use finchers::endpoint::ApplyError;
use finchers::prelude::*;
use finchers::rt::test;
use futures::future;
use http::{Request, Response};

#[test]
fn test_recover() {
    let mut runner = test::runner({
        let endpoint = syntax::verb::get()
            .and(syntax::segment("posts"))
            .and(syntax::param::<u32>())
            .and(syntax::eos())
            .map(|id: u32| format!("param={}", id))
            .with_output::<(String,)>();

        let recovered =
            endpoint
                .wrap(endpoint::wrapper::or_reject())
                .wrap(endpoint::wrapper::recover(|err| {
                    if err.is::<ApplyError>() {
                        future::ok(
                            Response::builder()
                                .status(err.status_code())
                                .body(err.status_code().to_string())
                                .expect("should be a valid response"),
                        )
                    } else {
                        future::err(err)
                    }
                }));

        recovered
    });

    assert!(runner.apply(Request::get("/posts/42")).is_ok());
    assert!(runner.apply(Request::get("/posts/")).is_ok());
    assert!(runner.apply(Request::post("/posts/42")).is_ok());
    assert!(runner.apply(Request::get("/posts/foo")).is_ok());
}
