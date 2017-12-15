use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::iter::FromIterator;

use hyper;
use url::form_urlencoded;

use request::{self, Body, FromParam, Request};


#[doc(hidden)]
#[derive(Debug)]
pub struct RequestInfo {
    /// The information of incoming HTTP request, without the request body
    request: Request,

    /// The stream of request body
    body: RefCell<Option<Body>>,

    /// A HashMap contains parsed result of query parameters
    queries: HashMap<String, Vec<String>>,

    /// The elements of path segments.
    routes: Vec<String>,
}

impl RequestInfo {
    pub fn new(request: Request, body: Body) -> Self {
        let body = RefCell::new(Some(body));
        let routes = to_path_segments(request.path());
        let queries = request.query().map(to_query_map).unwrap_or_default();
        Self {
            request,
            body,
            routes,
            queries,
        }
    }
}


/// A set of values, contains the incoming HTTP request and the finchers-specific context.
#[derive(Debug, Clone)]
pub struct Context {
    inner: Rc<RequestInfo>,
    pos: usize,
}

impl Context {
    pub(crate) fn from_hyper(req: hyper::Request) -> Self {
        let (req, body) = request::reconstruct(req);
        let info = RequestInfo::new(req, body);
        Self::new(info)
    }

    pub(crate) fn new(inner: RequestInfo) -> Self {
        Context {
            inner: Rc::new(inner),
            pos: 0,
        }
    }

    /// Return the reference of `Request`
    pub fn request(&self) -> &Request {
        &self.inner.request
    }

    /// Take and return the instance of request body, if available.
    pub fn take_body(&mut self) -> Option<Body> {
        self.inner.body.borrow_mut().take()
    }

    /// Pop and return the front element of path segments.
    pub fn next_segment(&mut self) -> Option<&str> {
        if self.pos >= self.inner.routes.len() {
            return None;
        }
        let pos = self.pos;
        self.pos += 1;
        Some(&self.inner.routes[pos])
    }

    /// Collect and return the remaining path segments, if available
    pub fn collect_remaining_segments<I, T>(&mut self) -> Option<Result<I, T::Error>>
    where
        I: FromIterator<T>,
        T: FromParam,
    {
        if self.pos >= self.inner.routes.len() {
            return None;
        }
        let pos = self.pos;
        self.pos = self.inner.routes.len();
        Some(
            self.inner.routes[pos..]
                .into_iter()
                .map(|s| T::from_param(s))
                .collect(),
        )
    }

    /// Count the length of remaining path segments
    pub fn count_remaining_segments(&mut self) -> usize {
        self.inner.routes.len() - self.pos
    }

    /// Return the first value of the query parameter whose name is `name`, if exists
    pub fn query<S: AsRef<str>>(&self, name: S) -> Option<&str> {
        self.inner
            .queries
            .get(name.as_ref())
            .and_then(|q| q.get(0).map(|s| &*s as &str))
    }

    /// Returns all query parameters with name `name`
    pub fn queries<S: AsRef<str>>(&self, name: S) -> Vec<&str> {
        self.inner
            .queries
            .get(name.as_ref())
            .map(|q| q.iter().map(|s| &*s as &str).collect())
            .unwrap_or_default()
    }
}


fn to_path_segments(s: &str) -> Vec<String> {
    s.trim_left_matches("/")
        .split("/")
        .filter(|s| s.trim() != "")
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod to_path_segments_test {
    use super::to_path_segments;

    #[test]
    fn case1() {
        assert_eq!(to_path_segments("/"), &[] as &[&str]);
    }

    #[test]
    fn case2() {
        assert_eq!(to_path_segments("/foo"), &["foo"]);
    }

    #[test]
    fn case3() {
        assert_eq!(to_path_segments("/foo/bar/"), &["foo", "bar"]);
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
