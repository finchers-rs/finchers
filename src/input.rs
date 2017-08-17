use hyper::{Request, Body, Method};

#[derive(Debug)]
pub struct Input {
    pub req: Request<Body>,
}

impl Input {
    pub fn new(method: Method, uri: &str) -> Input {
        let uri = uri.parse().unwrap();
        Input { req: Request::new(method, uri) }
    }
}
