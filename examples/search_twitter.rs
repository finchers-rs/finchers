extern crate egg_mode;
#[macro_use]
extern crate finchers;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

use finchers::contrib::urlencoded::{self, queries, FromUrlEncoded, UrlDecodeError};
use finchers::http::{IntoResponse, Response};
use finchers::errors::StdErrorResponseBuilder;
use finchers::responder::DefaultResponder;
use finchers::service::FinchersService;
use finchers::http::header;

use std::rc::Rc;
use futures::{Future, Stream};
use hyper::server::Http;
use tokio_core::reactor::Core;

#[derive(Debug)]
struct SearchParams {
    keyword: String,
    count: u32,
}

impl FromUrlEncoded for SearchParams {
    fn from_urlencoded(iter: urlencoded::Parse) -> Result<Self, UrlDecodeError> {
        let mut keyword = None;
        let mut count = None;
        for (key, value) in iter {
            match &key as &str {
                "keyword" => keyword = Some(value.into_owned()),
                "count" => count = Some(value.parse().map_err(|e| UrlDecodeError::other(e))?),
                s => return Err(UrlDecodeError::invalid_key(s.to_owned())),
            }
        }

        Ok(SearchParams {
            keyword: keyword.ok_or_else(|| UrlDecodeError::missing_key("keyword"))?,
            count: count.ok_or_else(|| UrlDecodeError::missing_key("count"))?,
        })
    }
}

#[derive(Debug, Serialize)]
struct HandleResult {
    statuses: Vec<String>,
}

impl IntoResponse for HandleResult {
    fn into_response(self) -> Response {
        let body = serde_json::to_vec(&self).unwrap();
        Response::new()
            .with_header(header::ContentType::json())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }
}

#[derive(Debug)]
struct HandleError(egg_mode::error::Error);

impl IntoResponse for HandleError {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::server_error(self.0).finish()
    }
}

fn read_line(message: &str) -> String {
    print!("\n{}:\n", message);
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line
}

fn retrieve_access_token(core: &mut Core) -> egg_mode::Token {
    let consumer_key = read_line("Consumer Key").trim().to_string();
    let consumer_secret = read_line("Consume Secret").trim().to_string();
    let consumer_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

    let handle = core.handle();
    let request_token = core.run(egg_mode::request_token(&consumer_token, "oob", &handle))
        .unwrap();

    println!();
    println!("Open the following URL and retrieve the PIN");
    println!("{}", egg_mode::authorize_url(&request_token));
    println!();

    let pin = read_line("PIN");

    let (access_token, _user_id, username) = core.run(egg_mode::access_token(
        consumer_token,
        &request_token,
        pin,
        &handle,
    )).unwrap();

    println!();
    println!("Logged in as @{}", username);

    access_token
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let service = {
        let token = retrieve_access_token(&mut core);

        let endpoint = {
            use finchers::Endpoint;
            Rc::new(endpoint!("search").with(queries::<SearchParams>()))
        };

        let handler = Rc::new({
            let handle = handle.clone();
            move |params: SearchParams| {
                egg_mode::search::search(params.keyword)
                    .result_type(egg_mode::search::ResultType::Recent)
                    .count(params.count)
                    .call(&token, &handle)
                    .map_err(HandleError)
                    .map(
                        |egg_mode::Response {
                             response: search, ..
                         }| {
                            HandleResult {
                                statuses: search
                                    .statuses
                                    .into_iter()
                                    .map(|tweet| tweet.text)
                                    .collect(),
                            }
                        },
                    )
            }
        });

        FinchersService::new(endpoint, handler, DefaultResponder::default())
    };

    let addr = "0.0.0.0:4000".parse().unwrap();
    let serves = Http::new()
        .pipeline(true)
        .serve_addr_handle(&addr, &core.handle(), move || Ok(service.clone()))
        .unwrap();

    let _ = core.run(serves.for_each(|conn| conn.map(|_| ())));
}
