#[macro_use]
extern crate error_chain;
extern crate finchers;

use finchers::{Context, Endpoint, EndpointError};
use finchers::endpoint::method::get;
use finchers::endpoint::param;
use finchers::response::{Responder, Response, ResponseBuilder, StatusCode};
use finchers::ServerBuilder;

error_chain! {
    foreign_links {
        Endpoint(EndpointError);
    }
}

impl Responder for Error {
    fn respond_to(&mut self, _: &mut Context) -> Response {
        ResponseBuilder::default()
            .status(StatusCode::NotFound)
            .finish()
    }
}

fn main() {
    // GET /foo/:id/bar
    let endpoint = get(("foo", param(), "bar"))
        .map(|(_, name, _)| name)
        .and_then(|name: String| -> Result<_> { Ok(format!("Hello, {}", name)) });

    ServerBuilder::default()
        .bind("0.0.0.0:8080")
        .num_workers(1)
        .run_http(endpoint);
}
