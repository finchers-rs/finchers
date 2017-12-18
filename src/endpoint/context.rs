use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Component, Components, Path};
use std::rc::Rc;
use std::str::FromStr;
use url::form_urlencoded;
use request::Request;



#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    request: &'a Request,
    routes: Option<Components<'a>>,
    queries: Rc<Option<HashMap<Cow<'a, str>, Vec<Cow<'a, str>>>>>,
}

impl<'a> EndpointContext<'a> {
    pub fn new(request: &'a Request) -> Self {
        let mut routes = Path::new(request.path()).components();
        routes.next(); // skip the root ("/")
        let queries = request.query().map(parse_queries);
        EndpointContext {
            request,
            routes: Some(routes),
            queries: Rc::new(queries),
        }
    }

    pub fn request(&self) -> &Request {
        self.request
    }

    pub fn next_segment(&mut self) -> Option<&str> {
        self.routes.as_mut().and_then(|r| {
            r.next().map(|c| match c {
                Component::Normal(s) => s.to_str().unwrap(),
                _ => panic!("relatative path is not supported"),
            })
        })
    }

    pub fn collect_remaining_segments<I, T>(&mut self) -> Option<Result<I, T::Err>>
    where
        I: FromIterator<T>,
        T: FromStr,
    {
        let routes = self.routes.take()?;
        Some(
            routes
                .map(|c| match c {
                    Component::Normal(s) => s.to_str().unwrap().parse(),
                    _ => panic!("relative path is not supported"),
                })
                .collect(),
        )
    }

    pub fn count_remaining_segments(&mut self) -> usize {
        self.routes.take().map_or(0, |routes| routes.count())
    }

    pub fn query<S: AsRef<str>>(&mut self, name: S) -> Option<&str> {
        let queries = (*self.queries).as_ref()?;
        queries
            .get(name.as_ref())
            .and_then(|q| q.get(0).map(|s| &*s as &str))
    }

    pub fn queries<S: AsRef<str>>(&mut self, name: S) -> Option<Vec<&str>> {
        let queries = (*self.queries).as_ref()?;
        queries
            .get(name.as_ref())
            .map(|q| q.iter().map(|s| &*s as &str).collect())
    }
}



fn parse_queries(s: &str) -> HashMap<Cow<str>, Vec<Cow<str>>> {
    let mut queries = HashMap::new();
    for (key, value) in form_urlencoded::parse(s.as_bytes()) {
        queries.entry(key).or_insert(Vec::new()).push(value);
    }
    queries
}
