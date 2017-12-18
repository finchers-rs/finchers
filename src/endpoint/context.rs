use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Component, Components, Path};
use std::rc::Rc;
use std::str::FromStr;
use url::form_urlencoded;
use request::Request;


/// A set of values, contains the incoming HTTP request and the finchers-specific context.
#[derive(Debug, Clone)]
pub struct EndpointContext<'a> {
    request: &'a Request,
    routes: Option<Components<'a>>,
    queries: Rc<Option<HashMap<Cow<'a, str>, Vec<Cow<'a, str>>>>>,
}

impl<'a> EndpointContext<'a> {
    #[allow(missing_docs)]
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

    #[allow(missing_docs)]
    pub fn request(&self) -> &Request {
        self.request
    }

    /// Pop and return the front element of path segments.
    pub fn next_segment(&mut self) -> Option<&str> {
        self.routes.as_mut().and_then(|r| {
            r.next().map(|c| match c {
                Component::Normal(s) => s.to_str().unwrap(),
                _ => panic!("relatative path is not supported"),
            })
        })
    }

    /// Collect and return the remaining path segments, if available
    pub fn collect_remaining_segments<I, T>(&mut self) -> Option<Result<I, T::Error>>
    where
        I: FromIterator<T>,
        T: FromParam,
    {
        let routes = self.routes.take()?;
        Some(
            routes
                .map(|c| match c {
                    Component::Normal(s) => T::from_param(s.to_str().unwrap()),
                    _ => panic!("relative path is not supported"),
                })
                .collect(),
        )
    }

    /// Count the length of remaining path segments
    pub fn count_remaining_segments(&mut self) -> usize {
        self.routes.take().map_or(0, |routes| routes.count())
    }

    /// Return the first value of the query parameter whose name is `name`, if exists
    pub fn query<S: AsRef<str>>(&mut self, name: S) -> Option<&str> {
        let queries = (*self.queries).as_ref()?;
        queries
            .get(name.as_ref())
            .and_then(|q| q.get(0).map(|s| &*s as &str))
    }

    /// Returns all query parameters with name `name`
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


/// Represents the conversion from a path segment
pub trait FromParam: Sized {
    /// The error type of `from_param()`
    type Error;

    /// Try to convert a `str` to itself
    fn from_param(s: &str) -> Result<Self, Self::Error>;
}

macro_rules! impl_from_param {
    ($($t:ty),*) => {$(
        impl FromParam for $t {
            type Error = <$t as FromStr>::Err;

            fn from_param(s: &str) -> Result<Self, Self::Error> {
                s.parse()
            }
        }
    )*}
}

impl_from_param!(
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    isize,
    usize,
    f32,
    f64,
    String
);
