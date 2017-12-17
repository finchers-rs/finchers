use std::iter::FromIterator;
use request::{FromParam, RequestInfo};


/// A set of values, contains the incoming HTTP request and the finchers-specific context.
#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    request: &'a RequestInfo,
    routes: Vec<String>,
    pos: usize,
}

impl<'a> EndpointContext<'a> {
    #[allow(missing_docs)]
    pub fn new(request: &'a RequestInfo) -> Self {
        let routes = to_path_segments(request.request().path());
        EndpointContext {
            request,
            routes,
            pos: 0,
        }
    }

    #[allow(missing_docs)]
    pub fn request(&self) -> &RequestInfo {
        &self.request
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
