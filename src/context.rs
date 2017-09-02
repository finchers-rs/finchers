use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use url::form_urlencoded;
use std::iter::FromIterator;
use std::slice::Iter;

use request::{Body, Request};
use endpoint::path::FromPath;


#[doc(hidden)]
#[derive(Debug)]
pub struct RequestInfo<'r> {
    /// The information of incoming HTTP request, without the request body
    request: &'r Request,

    /// The stream of request body
    body: RefCell<Option<Body>>,

    /// A HashMap contains parsed result of query parameters
    queries: HashMap<Cow<'r, str>, Vec<Cow<'r, str>>>,

    /// The elements of path segments.
    routes: Vec<&'r str>,
}

impl<'r> RequestInfo<'r> {
    pub fn new(request: &'r Request, body: Body) -> Self {
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
pub struct Context<'b, 'r: 'b> {
    inner: &'b RequestInfo<'r>,
    /// An iterator of remaining path segments in the context.
    routes: Option<Iter<'b, &'r str>>,
}

impl<'r, 'b> From<&'b RequestInfo<'r>> for Context<'r, 'b> {
    fn from(base: &'b RequestInfo<'r>) -> Self {
        Context {
            inner: base,
            routes: Some(base.routes.iter()),
        }
    }
}

impl<'r, 'b> Context<'r, 'b> {
    /// Return the reference of `Request`
    pub fn request(&self) -> &'r Request {
        &self.inner.request
    }

    /// Take and return the instance of request body, if available.
    pub fn take_body(&mut self) -> Option<Body> {
        self.inner.body.borrow_mut().take()
    }

    /// Pop and return the front element of path segments.
    pub fn next_segment(&mut self) -> Option<&str> {
        self.routes
            .as_mut()
            .and_then(|routes| routes.next().map(|s| *s))
    }

    /// Collect and return the remaining path segments, if available
    pub fn collect_remaining_segments<I, T>(&mut self) -> Option<Option<I>>
    where
        I: FromIterator<T>,
        T: FromPath,
    {
        self.routes
            .take()
            .map(|routes| routes.map(|s| T::from_path(s)).collect())
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
