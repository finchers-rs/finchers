use finchers::output::body::optional;
use finchers::prelude::*;
use finchers::server::middleware::map_response_body;

use http::Method;
use tower_web::middleware::cors::{AllowedOrigins, CorsBuilder};
use tower_web::middleware::log::LogMiddleware;

fn main() {
    let endpoint = endpoint::cloned("Hello, world!");

    let log_middleware = LogMiddleware::new(module_path!());
    let cors_middleware = CorsBuilder::new()
        .allow_origins(AllowedOrigins::Any { allow_null: true })
        .allow_methods(&[Method::GET])
        .build();

    println!("Listening on http://127.0.0.1:4000");
    finchers::server::start(endpoint)
        .with_tower_middleware(log_middleware)
        .with_tower_middleware(cors_middleware)
        .with_middleware(map_response_body(optional))
        .serve("127.0.0.1:4000")
        .expect("failed to start the server");
}
