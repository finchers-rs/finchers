extern crate bytes;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate finchers;
extern crate futures;
extern crate http;
#[macro_use]
extern crate matches;
extern crate mime;
#[macro_use]
extern crate serde;
#[cfg(feature = "tower-web")]
extern crate tower_web;

mod endpoint;
mod endpoints;

#[test]
fn smoketest_new_runtime() {
    use finchers::prelude::*;
    drop(|| finchers::server::start(endpoint::cloned("Hello")).serve("127.0.0.1:4000"))
}

#[cfg(feature = "tower-web")]
#[test]
fn smoketest_tower_web_middlewares() {
    use finchers::output::body::optional;
    use finchers::prelude::*;
    use finchers::server::middleware::map_response_body;
    use tower_web::middleware::log::LogMiddleware;

    drop(|| {
        finchers::server::start(endpoint::unit())
            .with_tower_middleware(LogMiddleware::new(module_path!()))
            .with_middleware(map_response_body(Some))
            .with_middleware(map_response_body(optional))
            .serve("127.0.0.1:4000")
    });
}

#[test]
fn test_perform_on_error_response() {
    use finchers::prelude::*;
    use finchers::test;

    let mut runner =
        test::runner({ endpoint::lazy(|| Err::<&str, _>(finchers::error::bad_request("error"))) });

    let response = runner.perform("/").unwrap();
    assert_eq!(response.status().as_u16(), 400);
}
