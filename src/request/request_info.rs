#![allow(missing_docs)]

use std::cell::RefCell;
use std::collections::HashMap;
use hyper;
use url::form_urlencoded;
use request::{self, Body, Request};


#[derive(Debug)]
pub struct RequestInfo {
    /// The information of incoming HTTP request, without the request body
    request: Request,

    /// The stream of request body
    body: RefCell<Option<Body>>,

    /// A HashMap contains parsed result of query parameters
    queries: HashMap<String, Vec<String>>,
}

impl RequestInfo {
    pub fn new(request: Request, body: Body) -> Self {
        let body = RefCell::new(Some(body));
        let queries = request.query().map(to_query_map).unwrap_or_default();
        Self {
            request,
            body,
            queries,
        }
    }

    pub fn from_hyper(req: hyper::Request) -> Self {
        let (req, body) = request::reconstruct(req);
        Self::new(req, body)
    }

    /// Return the reference of `Request`
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// Take and return the instance of request body, if available.
    pub fn take_body(&self) -> Option<Body> {
        self.body.borrow_mut().take()
    }

    /// Return the first value of the query parameter whose name is `name`, if exists
    pub fn query<S: AsRef<str>>(&self, name: S) -> Option<&str> {
        self.queries
            .get(name.as_ref())
            .and_then(|q| q.get(0).map(|s| &*s as &str))
    }

    /// Returns all query parameters with name `name`
    pub fn queries<S: AsRef<str>>(&self, name: S) -> Vec<&str> {
        self.queries
            .get(name.as_ref())
            .map(|q| q.iter().map(|s| &*s as &str).collect())
            .unwrap_or_default()
    }
}

fn to_query_map(s: &str) -> HashMap<String, Vec<String>> {
    let mut queries = HashMap::new();
    for (key, value) in form_urlencoded::parse(s.as_bytes()) {
        queries
            .entry(key.into_owned())
            .or_insert(Vec::new())
            .push(value.into_owned());
    }
    queries
}
