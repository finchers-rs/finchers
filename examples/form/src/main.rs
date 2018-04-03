#[macro_use]
extern crate finchers;
extern crate http;
#[macro_use]
extern crate serde_derive;

use finchers::prelude::*;
use finchers::endpoint::prelude::*;
use finchers::service::{backend, Server};
use finchers::urlencoded::{form_body, from_csv, queries};
use std::fmt;

#[derive(Debug, Deserialize, HttpStatus)]
pub struct FormParam {
    query: String,
    count: Option<usize>,
    #[serde(deserialize_with = "from_csv")]
    tags: Option<Vec<String>>,
}

impl fmt::Display for FormParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#}", self)
    }
}

fn main() {
    let endpoint = endpoint("search")
        .with(choice![get(queries()), post(form_body()),])
        .map(|param: FormParam| {
            println!("Received: {:#}", param);
            param
        });

    let service = endpoint.into_service::<FormParam>();
    Server::from_service(service).run(backend::default());
}
