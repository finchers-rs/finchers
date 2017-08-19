extern crate finchers;
extern crate futures;
extern crate hyper;

use finchers::Endpoint;
use finchers::combinator::method::post;
use finchers::combinator::path::{string_, end_};
use finchers::combinator::param::param;
use finchers::combinator::body::take_body;
use finchers::response::Json;

use futures::{Future, Stream};

fn main() {
    let new_endpoint = || {
        post("hello".with(string_).skip(end_).join3(
            param::<String>("foo"),
            take_body(),
        )).map(|(name, param, body)| {
            let body: String = body.fold(Vec::new(), |mut body,
             chunk|
             -> Result<Vec<u8>, hyper::Error> {
                body.extend_from_slice(&chunk);
                Ok(body)
            }).map(|body| unsafe { String::from_utf8_unchecked(body) })
                .wait()
                .ok()
                .expect("failed to read body");
            Json(vec![format!("Hello, {}, {}", name, param), body])
        })
    };

    finchers::server::run_http(new_endpoint, "127.0.0.1:3000");
}
