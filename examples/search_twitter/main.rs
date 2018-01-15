extern crate egg_mode;
#[macro_use]
extern crate finchers;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

mod common;
mod handler;

use finchers::contrib::urlencoded::queries;
use finchers::responder::DefaultResponder;
use finchers::service::FinchersService;

use std::rc::Rc;
use futures::{Future, Stream};
use hyper::server::Http;
use tokio_core::reactor::Core;

use handler::SearchTwitterHandler;

#[derive(Debug)]
struct SearchParams {
    keyword: String,
    count: u32,
}

mod __impl_search_params {
    use finchers::contrib::urlencoded::{FromUrlEncoded, Parse, UrlDecodeError};
    use super::SearchParams;

    impl FromUrlEncoded for SearchParams {
        fn from_urlencoded(iter: Parse) -> Result<Self, UrlDecodeError> {
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
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let service = {
        use finchers::Endpoint;
        let endpoint = Rc::new(endpoint!("search").with(queries()));
        let token = common::retrieve_access_token(&mut core);
        let handler = SearchTwitterHandler::new(token, &handle);
        FinchersService::new(endpoint, handler, DefaultResponder::default())
    };

    let addr = "0.0.0.0:4000".parse().unwrap();
    let serves = Http::new()
        .pipeline(true)
        .serve_addr_handle(&addr, &core.handle(), move || Ok(service.clone()))
        .unwrap();
    let _ = core.run(serves.for_each(|conn| conn.map(|_| ())));
}
