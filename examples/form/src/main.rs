#[macro_use]
extern crate finchers;
extern crate http;
#[macro_use]
extern crate serde_derive;

use finchers::Endpoint;
use finchers::output::Debug;
use finchers::urlencoded::{from_csv, queries, Form};

#[derive(Debug, Deserialize, HttpStatus)]
pub struct FormParam {
    query: String,
    count: Option<usize>,
    #[serde(deserialize_with = "from_csv")]
    tags: Option<Vec<String>>,
}

fn endpoint() -> impl Endpoint<Item = Debug> + Send + Sync + 'static {
    use finchers::endpoint::prelude::*;

    path("search")
        .right(choice![get(queries()), post(body()).map(|Form(body)| body),])
        .inspect(|param: &FormParam| {
            println!("Received: {:?}", param);
        })
        .map(|param| Debug::new(param).pretty(true))
}

fn main() {
    finchers::run(endpoint());
}
