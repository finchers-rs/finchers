#![allow(missing_docs)]

use std::borrow::Cow;
use std::fmt;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::str::FromStr;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};

#[derive(Clone)]
pub struct MatchPath<E> {
    kind: MatchPathKind,
    _marker: PhantomData<fn() -> E>,
}

impl<E> fmt::Debug for MatchPath<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MatchPath")
            .field("kind", &self.kind)
            .finish()
    }
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

pub fn path<T: FromStr>() -> ExtractPath<T> {
    ExtractPath {
        _marker: PhantomData,
    }
}

pub struct ExtractPath<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> fmt::Debug for ExtractPath<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPath").finish()
    }
}

impl<T: FromStr> Endpoint for ExtractPath<T> {
    type Item = T;
    type Error = T::Err;
    type Task = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        ctx.segments().next().map(|s| s.parse())
    }
}

pub fn paths<I, T>() -> ExtractPaths<I, T>
where
    I: FromIterator<T>,
    T: FromStr,
{
    ExtractPaths {
        _marker: PhantomData,
    }
}

pub struct ExtractPaths<I, T> {
    _marker: PhantomData<fn() -> (I, T)>,
}

impl<I, T> fmt::Debug for ExtractPaths<I, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPaths").finish()
    }
}

impl<I, T> Endpoint for ExtractPaths<I, T>
where
    I: FromIterator<T>,
    T: FromStr,
{
    type Item = I;
    type Error = T::Err;
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
        match runner.run(request) {
            Some(Ok(())) => (),
            _ => panic!("does not match"),
        }
    }

    #[test]
    fn test_endpoint_reject_path() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("bar");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        assert!(runner.run(request).is_none());
    }

    #[test]
    fn test_endpoint_match_multi_segments() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("/foo/bar");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo/bar".parse().unwrap());
        match runner.run(request) {
            Some(Ok(())) => (),
            _ => panic!("does not match"),
        }
    }

    #[test]
    fn test_endpoint_reject_multi_segments() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("/foo/bar");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo/baz".parse().unwrap());
        assert!(runner.run(request).is_none());
    }

    #[test]
    fn test_endpoint_reject_short_path() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("/foo/bar/baz");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo/bar".parse().unwrap());
        assert!(runner.run(request).is_none());
    }

    #[test]
    fn test_endpoint_match_all_path() {
        let endpoint = IntoEndpoint::<(), ()>::into_endpoint("*");
        let mut runner = TestRunner::new(endpoint).unwrap();

        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        match runner.run(request) {
            Some(Ok(())) => (),
            _ => panic!("does not match"),
        }
    }

    #[test]
    fn test_endpoint_extract_integer() {
        let endpoint = path::<i32>();
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/42".parse().unwrap());
        match runner.run(request) {
            Some(Ok(42)) => (),
            _ => panic!("does not match"),
        }
    }

    #[test]
    fn test_endpoint_extract_wrong_integer() {
        let endpoint = path::<i32>();
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/foo".parse().unwrap());
        match runner.run(request) {
            Some(Err(..)) => (),
            _ => panic!("does not match"),
        }
    }

    #[test]
    fn test_endpoint_extract_strings() {
        let endpoint = paths::<Vec<String>, String>();
        let mut runner = TestRunner::new(endpoint).unwrap();
        let request = Request::new(Method::Get, "/foo/bar".parse().unwrap());
        match runner.run(request) {
            Some(Ok(paths)) => {
                assert_eq!(paths, vec!["foo".to_string(), "bar".to_string()]);
            }
            _ => panic!("does not match"),
        }
    }
}
