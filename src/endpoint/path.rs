use std::borrow::Cow;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::str::FromStr;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchPath<E> {
    kind: MatchPathKind,
    _marker: PhantomData<fn() -> E>,
}

#[derive(Debug, Clone, PartialEq)]
enum MatchPathKind {
    Segments(Vec<String>),
    AllSegments,
}
use self::MatchPathKind::*;

impl<E> Endpoint for MatchPath<E> {
    type Item = ();
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match self.kind {
            Segments(ref segments) => {
                let mut matched = true;
                for segment in segments {
                    matched = matched && *try_opt!(ctx.segments().next()) == *segment;
                }
                if matched {
                    Some(Ok(()))
                } else {
                    None
                }
            }
            AllSegments => {
                let _ = ctx.segments().count();
                Some(Ok(()))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseMatchError {
    EmptyString,
}

#[allow(missing_docs)]
pub fn match_<E>(s: &str) -> Result<MatchPath<E>, ParseMatchError> {
    let s = s.trim().trim_left_matches("/").trim_right_matches("/");
    let kind = if s == "*" {
        AllSegments
    } else {
        let mut segments = Vec::new();
        for segment in s.split("/").map(|s| s.trim()) {
            if segment.is_empty() {
                return Err(ParseMatchError::EmptyString);
            }
            segments.push(segment.into());
        }
        Segments(segments)
    };

    Ok(MatchPath {
        kind,
        _marker: PhantomData,
    })
}

impl<'a, E> IntoEndpoint<(), E> for &'a str {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(self).unwrap()
    }
}

impl<E> IntoEndpoint<(), E> for String {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(&self).unwrap()
    }
}

impl<'a, E> IntoEndpoint<(), E> for Cow<'a, str> {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(&*self).unwrap()
    }
}

#[allow(missing_docs)]
pub fn path<T: FromStr, E>() -> ExtractPath<T, E> {
    ExtractPath(PhantomData)
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ExtractPath<T, E>(PhantomData<fn() -> (T, E)>);

impl<T, E> Endpoint for ExtractPath<T, E>
where
    T: FromStr,
    E: From<T::Err>,
{
    type Item = T;
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        ctx.segments().next().map(|s| s.parse().map_err(Into::into))
    }
}

#[allow(missing_docs)]
pub fn paths<I, T, E>() -> ExtractPaths<I, T, E>
where
    I: FromIterator<T>,
    T: FromStr,
{
    ExtractPaths(PhantomData)
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ExtractPaths<I, T, E>(PhantomData<fn() -> (I, T, E)>);

impl<I, T, E> Endpoint for ExtractPaths<I, T, E>
where
    I: FromIterator<T>,
    T: FromStr,
    E: From<T::Err>,
{
    type Item = I;
    type Error = E;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        Some(
            ctx.segments()
                .map(|s| s.parse().map_err(Into::into))
                .collect::<Result<_, _>>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::ParseIntError;
    use std::string::ParseError;
    use hyper::{Method, Request};
    use test::TestRunner;

    #[test]
    fn test_match_single_segment() {
        assert_eq!(
            match_::<()>("foo").map(|m| m.kind),
            Ok(Segments(vec!["foo".to_owned()]))
        );
    }

    #[test]
    fn test_match_multi_segments() {
        assert_eq!(
            match_::<()>("foo/bar").map(|m| m.kind),
            Ok(Segments(vec!["foo".to_owned(), "bar".to_owned()]))
        );
    }

    #[test]
    fn test_match_all_segments() {
        assert_eq!(match_::<()>("*").map(|m| m.kind), Ok(AllSegments));
    }

    #[test]
    fn test_match_failure_empty() {
        assert_eq!(
            match_::<()>("").map(|m| m.kind),
            Err(ParseMatchError::EmptyString)
        );
    }

    #[test]
    fn test_match_failure_empty_2() {
        assert_eq!(
            match_::<()>("foo//bar").map(|m| m.kind),
            Err(ParseMatchError::EmptyString)
        );
    }

    #[test]
    fn test_endpoint_match_path() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("foo");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        assert_eq!(runner.run(request), Some(Ok(())));
    }

    #[test]
    fn test_endpoint_reject_path() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("bar");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        assert_eq!(runner.run(request), None);
    }

    #[test]
    fn test_endpoint_match_multi_segments() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("/foo/bar");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo/bar".parse().unwrap());
        assert_eq!(runner.run(request), Some(Ok(())));
    }

    #[test]
    fn test_endpoint_reject_multi_segments() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("/foo/bar");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo/baz".parse().unwrap());
        assert_eq!(runner.run(request), None);
    }

    #[test]
    fn test_endpoint_reject_short_path() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("/foo/bar/baz");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo/bar".parse().unwrap());
        assert_eq!(runner.run(request), None);
    }

    #[test]
    fn test_endpoint_match_all_path() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("*");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        assert_eq!(runner.run(request), Some(Ok(())));
    }

    #[test]
    fn test_endpoint_extract_integer() {
        let endpoint = path::<i32, ParseIntError>();
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/42".parse().unwrap());
        assert_eq!(runner.run(request), Some(Ok(42)));
    }

    #[test]
    fn test_endpoint_extract_wrong_integer() {
        let endpoint = path::<i32, ParseIntError>();
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        assert_eq!(runner.run(request).map(|r| r.is_err()), Some(true));
    }

    #[test]
    fn test_endpoint_extract_strings() {
        let endpoint = paths::<Vec<String>, String, ParseError>();
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/foo/bar".parse().unwrap());
        assert_eq!(
            runner.run(request),
            Some(Ok(vec!["foo".to_string(), "bar".to_string()]))
        );
    }
}
