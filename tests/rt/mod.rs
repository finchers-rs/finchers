#![cfg(feature = "rt")]

mod endpoint;
mod endpoints;

use finchers;

#[test]
fn smoketest_new_runtime() {
    use finchers::prelude::*;
    drop(|| finchers::rt::launch(endpoint::cloned("Hello")).serve_http("127.0.0.1:4000"))
}

#[cfg(feature = "tower-web")]
#[test]
fn smoketest_tower_web_middlewares() {
    use finchers::output::body::optional;
    use finchers::prelude::*;
    use finchers::rt::middleware::map_response_body;
    use tower_web::middleware::log::LogMiddleware;

    drop(|| {
        finchers::rt::launch(endpoint::unit())
            .with_tower_middleware(LogMiddleware::new(module_path!()))
            .with_middleware(map_response_body(Some))
            .with_middleware(map_response_body(optional))
            .serve_http("127.0.0.1:4000")
    });
}
