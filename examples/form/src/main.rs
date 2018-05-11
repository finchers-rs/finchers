#[macro_use]
extern crate finchers;
#[macro_use]
extern crate serde;

use finchers::Endpoint;
use finchers::output::Debug;

fn endpoint() -> impl Endpoint<Output = Debug> + 'static {
    use finchers::endpoint::prelude::*;
    use finchers::endpoint::query::{from_csv, query, Form, Serde};

    #[derive(Debug, Deserialize, HttpResponse)]
    pub struct FormParam {
        query: String,
        count: Option<usize>,
        #[serde(deserialize_with = "from_csv", default)]
        tags: Option<Vec<String>>,
    }

    // Create an endpoint for parsing the form-urlencoded parameter in the request.
    let urlencoded_param = choice![
        // Parse the query string when GET request.
        get(query().unwrap_ok()).map(Serde::into_inner),
        // Parse the message body when POST request.
        post(body().unwrap_ok()).map(|Form(Serde(param))| param),
    ]
    // annotate to the endpoint that the inner type is FormParam.
    .as_t::<FormParam>();

    path("search")
        .right(urlencoded_param)
        .inspect(|param| println!("Received: {:?}", param))
        .map(|param| Debug::new(param).pretty(true))
}

fn main() {
    finchers::run(endpoint());
}
