use std::sync::Arc;
use std::collections::HashMap;
use hyper::{Request, Body, Method};
use url::form_urlencoded;

#[derive(Debug)]
pub struct Input {
    pub req: Arc<Request<Body>>,
    pub routes: Vec<String>,
    pub params: HashMap<String, String>,
}

impl Input {
    pub fn new(method: Method, uri: &str) -> Input {
        let uri = uri.parse().unwrap();
        let req = Request::new(method, uri);
        let routes = req.path()
            .trim_left_matches("/")
            .split("/")
            .filter(|s| s.trim() != "")
            .map(Into::into)
            .collect();
        let params = req.query()
            .map(|query| {
                form_urlencoded::parse(query.as_bytes())
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect()
            })
            .unwrap_or_default();
        Input {
            req: Arc::new(req),
            routes,
            params,
        }
    }
}
