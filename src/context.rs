use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use url::form_urlencoded;
use std::iter::FromIterator;
use std::slice::Iter;
use std::str::FromStr;

use request::{Body, Request};

/// A Finchers-specific context and the incoming HTTP request, without the request body
#[derive(Debug, Clone)]
pub struct Context<'r: 'p + 'q, 'b, 'p, 'q> {
    /// A reference of the incoming HTTP request, without the request body
    request: &'r Request,
    /// A reference of the request body
    body: &'b RefCell<Option<Body>>,
    /// A sequence of remaining path segments
    routes: Option<Iter<'p, &'r str>>,
    /// A map of parsed queries
    queries: &'q HashMap<Cow<'r, str>, Vec<Cow<'r, str>>>,
}

impl<'r, 'b, 'p, 'q> Context<'r, 'b, 'p, 'q> {
    /// Create an instance of `Context` from a reference of the incoming HTTP request
    pub(crate) fn new(
        request: &'r Request,
        body: &'b RefCell<Option<Body>>,
        routes: Iter<'p, &'r str>,
        queries: &'q HashMap<Cow<'r, str>, Vec<Cow<'r, str>>>,
    ) -> Self {
        Context {
            request,
            body,
            routes: Some(routes),
            queries,
        }
    }

    /// Return the reference of `Request`
    pub fn request(&self) -> &'r Request {
        &self.request
    }

    /// Take and return the instance of request body, if available.
    pub fn take_body(&mut self) -> Option<Body> {
        self.body.borrow_mut().take()
    }

    /// Pop and return the front element of path segments.
    pub fn next_segment(&mut self) -> Option<&str> {
        self.routes
            .as_mut()
            .and_then(|routes| routes.next().map(|s| *s))
    }

    /// Collect and return the remaining path segments, if available
    pub fn collect_remaining_segments<I, T>(&mut self) -> Option<Result<I, T::Err>>
    where
        I: FromIterator<T>,
        T: FromStr,
    {
        self.routes
            .take()
            .map(|routes| routes.map(|s| s.parse()).collect())
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

pub(crate) fn create_inner<'r>(
    req: &'r Request,
    body: Body,
) -> (
    RefCell<Option<Body>>,
    Vec<&'r str>,
    HashMap<Cow<'r, str>, Vec<Cow<'r, str>>>,
) {
    let body = RefCell::new(Some(body));
    let routes = to_path_segments(req.path());
    let params = req.query().map(to_query_map).unwrap_or_default();
    (body, routes, params)
}


fn to_path_segments<'t>(s: &'t str) -> Vec<&'t str> {
    s.trim_left_matches("/")
        .split("/")
        .filter(|s| s.trim() != "")
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


fn to_query_map<'t>(s: &'t str) -> HashMap<Cow<'t, str>, Vec<Cow<'t, str>>> {
    let mut queries = HashMap::new();
    for (key, value) in form_urlencoded::parse(s.as_bytes()) {
        queries.entry(key).or_insert(Vec::new()).push(value);
    }
    queries
}
