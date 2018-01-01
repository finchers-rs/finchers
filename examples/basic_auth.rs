extern crate finchers;

use std::fmt;
use finchers::{Application, Endpoint, Responder};
use finchers::endpoint::header_opt;
use finchers::endpoint::method::get;
use finchers::http::{Headers, StatusCode};
use finchers::http::header::{Authorization, Basic};

#[derive(Debug)]
struct Unauthorized(String);

impl Responder for Unauthorized {
    type Body = ();
    fn status(&self) -> StatusCode {
        StatusCode::Unauthorized
    }
    fn headers(&self, h: &mut Headers) {
        h.set_raw("WWW-Authenticate", format!("Basic realm=\"{}\"", self.0));
    }
}

fn main() {
    #[derive(Debug)]
    struct User(String);

    impl fmt::Display for User {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    let auth = header_opt().and_then(|h| match h {
        Some(Authorization(Basic { username, password })) => password
            .ok_or_else(|| Unauthorized("Empty password".to_string()))
            .and_then(|password| {
                if username == "Alice" && password == "wonderland" {
                    Ok(User(username.to_string()))
                } else {
                    Err(Unauthorized("Authorization error".to_string()))
                }
            }),
        None => Err(Unauthorized("Empty header".to_string())),
    });

    let endpoint = get("user").with(auth).map(|user| {
        println!("Got user: {}", user);
        user.to_string()
    });

    Application::from_endpoint(endpoint).run();
}
