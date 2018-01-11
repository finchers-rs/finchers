#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate finchers;

use finchers::Endpoint;
use finchers::Application;
use finchers::endpoint::{body, path};
use finchers::endpoint::method::{get, post};
use errors::*;

fn main() {
    // GET /hello/:id
    let endpoint1 =
        get(("hello", path())).and_then(|(_, name): (_, String)| -> Result<_> { Ok(format!("Hello, {}", name)) });

    // POST /hello [String] (Content-type: text/plain; charset=utf-8)
    let endpoint2 = post(("hello", body())).and_then(|(_, body): (_, String)| Ok(format!("Received: {}", body)));

    let endpoint = choice!(endpoint1, endpoint2);

    Application::from_endpoint(endpoint).run();
}

// TODO: code generation
mod errors {
    use finchers::ErrorResponder;
    use finchers::http::{StatusCode, StringBodyError};
    use std::string::ParseError;

    error_chain! {
        types { Error, ErrorKind, ResultExt, Result; }

        foreign_links {
            Path(ParseError);
            Body(StringBodyError);
        }
    }

    impl ErrorResponder for Error {
        fn status(&self) -> StatusCode {
            match *self.kind() {
                ErrorKind::Path(ref e) => e.status(),
                ErrorKind::Body(ref e) => e.status(),
                _ => StatusCode::InternalServerError,
            }
        }

        fn message(&self) -> Option<String> {
            match *self.kind() {
                ErrorKind::Path(ref e) => e.message(),
                ErrorKind::Body(ref e) => e.message(),
                ErrorKind::Msg(ref msg) => Some(format!("other error: {}", msg)),
                _ => Some("Unknown error".to_string()),
            }
        }
    }
}
