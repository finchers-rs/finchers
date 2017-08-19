use std::borrow::Cow;
use std::collections::{VecDeque, HashMap};
use url::form_urlencoded;

use request::Request;

#[derive(Debug, Clone)]
pub struct Context<'r> {
    pub request: &'r Request,
    pub routes: VecDeque<&'r str>,
    pub params: HashMap<Cow<'r, str>, Cow<'r, str>>,
}

impl<'a> Context<'a> {
    pub fn new(request: &'a Request) -> Self {
        let routes = to_path_segments(request.path());
        let params = request.query().map(to_query_map).unwrap_or_default();
        Context {
            request,
            routes,
            params,
        }
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
