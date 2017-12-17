use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::FromIterator;
use url::form_urlencoded;
use request::{Body, FromParam, Request};


/// A set of values, contains the incoming HTTP request and the finchers-specific context.
#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    request: &'a Request,
    routes: Vec<&'a str>,
    queries: HashMap<Cow<'a, str>, Vec<Cow<'a, str>>>,
    pos: usize,
}

impl<'a> EndpointContext<'a> {
    #[allow(missing_docs)]
    pub fn new(request: &'a Request) -> Self {
        let routes = to_path_segments(request.path());
        let queries = request.query().map(to_query_map).unwrap_or_default();
        EndpointContext {
            request,
            routes,
            queries,
            pos: 0,
        }
    }

    #[allow(missing_docs)]
    pub fn request(&self) -> &Request {
        self.request
    }

    #[deprecated]
    pub fn take_body(&self) -> Option<Body> {
        self.request.body.borrow_mut().take()
    }


    /// Pop and return the front element of path segments.
    pub fn next_segment(&mut self) -> Option<&str> {
        if self.pos >= self.routes.len() {
            return None;
        }
        let pos = self.pos;
        self.pos += 1;
        Some(&self.routes[pos])
    }

    /// Collect and return the remaining path segments, if available
    pub fn collect_remaining_segments<I, T>(&mut self) -> Option<Result<I, T::Error>>
    where
        I: FromIterator<T>,
        T: FromParam,
    {
        if self.pos >= self.routes.len() {
            return None;
        }
        let pos = self.pos;
        self.pos = self.routes.len();
        Some(
            self.routes[pos..]
                .into_iter()
                .map(|s| T::from_param(s))
                .collect(),
        )
    }

    /// Count the length of remaining path segments
    pub fn count_remaining_segments(&mut self) -> usize {
        self.routes.len() - self.pos
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


fn to_path_segments(s: &str) -> Vec<&str> {
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


fn to_query_map(s: &str) -> HashMap<Cow<str>, Vec<Cow<str>>> {
    let mut queries = HashMap::new();
    for (key, value) in form_urlencoded::parse(s.as_bytes()) {
        queries.entry(key).or_insert(Vec::new()).push(value);
    }
    queries
}
