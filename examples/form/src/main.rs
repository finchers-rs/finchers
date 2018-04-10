#[macro_use]
extern crate finchers;
extern crate http;
#[macro_use]
extern crate serde_derive;

use finchers::endpoint::prelude::*;
use finchers::output::Debug;
use finchers::prelude::*;
use finchers::runtime::Server;
use finchers::urlencoded::{from_csv, queries, Form};
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
    let endpoint = path("search")
        .right(choice![get(queries()), post(body()).map(|Form(body)| body),])
        .map(|param: FormParam| {
            println!("Received: {:#}", param);
            Debug::new(param)
        });

    let service = endpoint.into_service();
    Server::new(service).run();
}
