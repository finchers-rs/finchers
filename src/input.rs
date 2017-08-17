use std::sync::Arc;
use hyper::{Request, Body, Method};

#[derive(Debug)]
pub struct Input {
    pub req: Arc<Request<Body>>,
    pub routes: Vec<String>,
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
        Input {
            req: Arc::new(req),
            routes,
        }
    }
}
