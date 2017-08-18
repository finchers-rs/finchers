use std::collections::{HashMap, VecDeque};

use futures::Future;
use url::form_urlencoded;

use combinator::{With, Map, Skip};
use request::Request;

error_chain! {
    types {
        EndpointError, EndpointErrorKind, ResultExt, EndpointResult;
    }
    errors {
        NoRoute {
            description("no route")
            display("no route")
        }
    }
}


/// A trait represents the HTTP endpoint.
pub trait Endpoint: Sized {
    type Item;
    type Error;
    type Future: Future<Item = Self::Item, Error = Self::Error>;

    /// Run the endpoint.
    fn apply(self, req: &Request, ctx: Context) -> EndpointResult<(Context, Self::Future)>;


    fn join<E>(self, e: E) -> (Self, E)
    where
        E: Endpoint<Error = Self::Error>,
    {
        (self, e)
    }

    fn join3<E1, E2>(self, e1: E1, e2: E2) -> (Self, E1, E2)
    where
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
    {
        (self, e1, e2)
    }

    fn join4<E1, E2, E3>(self, e1: E1, e2: E2, e3: E3) -> (Self, E1, E2, E3)
    where
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
        E3: Endpoint<Error = Self::Error>,
    {
        (self, e1, e2, e3)
    }

    fn join5<E1, E2, E3, E4>(self, e1: E1, e2: E2, e3: E3, e4: E4) -> (Self, E1, E2, E3, E4)
    where
        E1: Endpoint<Error = Self::Error>,
        E2: Endpoint<Error = Self::Error>,
        E3: Endpoint<Error = Self::Error>,
        E4: Endpoint<Error = Self::Error>,
    {
        (self, e1, e2, e3, e4)
    }

    fn with<E>(self, e: E) -> With<Self, E>
    where
        E: Endpoint<Error = Self::Error>,
    {
        With(self, e)
    }

    fn skip<E>(self, e: E) -> Skip<Self, E>
    where
        E: Endpoint<Error = Self::Error>,
    {
        Skip(self, e)
    }

    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Item) -> U,
    {
        Map(self, f)
    }
}


#[derive(Debug, Clone)]
pub struct Context {
    pub routes: VecDeque<String>,
    pub params: HashMap<String, String>,
}

impl Context {
    pub fn new(req: &Request) -> Self {
        let routes = to_path_segments(req.path());
        let params = req.query().map(to_query_map).unwrap_or_default();
        Context { routes, params }
    }
}


fn to_path_segments(s: &str) -> VecDeque<String> {
    s.trim_left_matches("/")
        .split("/")
        .filter(|s| s.trim() != "")
        .map(Into::into)
        .collect()
}

#[cfg(test)]
mod to_path_segments_test {
    use super::to_path_segments;

    #[test]
    fn case1() {
        assert_eq!(to_path_segments("/"), &[] as &[String]);
    }

    #[test]
    fn case2() {
        assert_eq!(to_path_segments("/foo"), &["foo".to_owned()]);
    }

    #[test]
    fn case3() {
        assert_eq!(
            to_path_segments("/foo/bar/"),
            &["foo".to_owned(), "bar".to_owned()]
        );
    }
}


fn to_query_map(s: &str) -> HashMap<String, String> {
    form_urlencoded::parse(s.as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}
