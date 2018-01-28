extern crate finchers;

use finchers::Application;
use finchers::endpoint::ok;
use finchers::http::{IntoResponse, Response, StatusCode};
use finchers::errors::HttpError;

#[derive(Debug)]
enum Dummy {}
impl std::fmt::Display for Dummy {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
        unreachable!()
    }
}
impl std::error::Error for Dummy {
    fn description(&self) -> &str {
        unreachable!()
    }
}
impl HttpError for Dummy {
    fn status_code(&self) -> StatusCode {
        unreachable!()
    }
}
impl IntoResponse for Dummy {
    fn into_response(self) -> Response {
        unreachable!()
    }
}

fn main() {
    let endpoint = ok::<&str, Dummy>("Hello, Finchers");
    Application::from_endpoint(endpoint).run();
}
