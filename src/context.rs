use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{VecDeque, HashMap};
use url::form_urlencoded;

use request::{Request, Body};

/// A Finchers-specific context and the incoming HTTP request, without the request body
#[derive(Debug, Clone)]
pub struct Context<'r, 'b> {
    /// A reference of the incoming HTTP request
    pub request: &'r Request,
    /// A sequence of remaining path segments
    pub routes: VecDeque<&'r str>,
    /// A map of parsed queries
    pub params: HashMap<Cow<'r, str>, Cow<'r, str>>,

    #[allow(missing_docs)]
    body: &'b RefCell<Option<Body>>,
}

impl<'r, 'b> Context<'r, 'b> {
    /// Create an instance of `Context` from a reference of the incoming HTTP request
    pub fn new(request: &'r Request, body: &'b RefCell<Option<Body>>) -> Self {
        let routes = to_path_segments(request.path());
        let params = request.query().map(to_query_map).unwrap_or_default();
        Context {
            request,
            routes,
            params,
            body,
        }
    }

    #[allow(missing_docs)]
    pub fn take_body(&mut self) -> Option<Body> {
        self.body.borrow_mut().take()
    }
}


fn to_path_segments<'t>(s: &'t str) -> VecDeque<&'t str> {
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
        assert_eq!(
            to_path_segments("/foo/bar/"),
            &["foo".to_owned(), "bar".to_owned()]
        );
    }
}


fn to_query_map<'t>(s: &'t str) -> HashMap<Cow<'t, str>, Cow<'t, str>> {
    form_urlencoded::parse(s.as_bytes()).collect()
}
