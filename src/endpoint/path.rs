#![allow(missing_docs)]

use std::borrow::Cow;
use std::fmt;
use std::error::Error;
use std::marker::PhantomData;
use std::str::FromStr;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint, Segments};

pub trait FromSegments: Sized {
    type Err;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err>;
}

mod implementors {
    use std::path::PathBuf;
    use errors::NeverReturn;
    use super::*;

    impl<T: FromStr> FromSegments for Vec<T> {
        type Err = T::Err;

        fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
            segments.into_iter().map(|s| s.parse()).collect()
        }
    }

    impl FromSegments for String {
        type Err = NeverReturn;

        fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
            let s = segments.remaining_path().to_owned();
            let _ = segments.last();
            Ok(s)
        }
    }

    impl FromSegments for PathBuf {
        type Err = NeverReturn;

        fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
            let s = PathBuf::from(segments.remaining_path());
            let _ = segments.last();
            Ok(s)
        }
    }
}

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
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
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
    type Error = ExtractPathError<T>;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        ctx.segments()
            .next()
            .map(|s| s.parse().map_err(ExtractPathError))
    }
}

pub struct ExtractPathError<T: FromStr = ()>(pub T::Err);

impl<T: FromStr> fmt::Debug for ExtractPathError<T>
where
    T::Err: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PathError").field(&self.0).finish()
    }
}

impl<T: FromStr> fmt::Display for ExtractPathError<T>
where
    T::Err: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: FromStr> Error for ExtractPathError<T>
where
    T::Err: Error,
{
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.0)
    }
}

impl<T: FromStr> PartialEq for ExtractPathError<T>
where
    T::Err: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }
}

pub fn paths<T: FromSegments>() -> ExtractPaths<T> {
    ExtractPaths {
        _marker: PhantomData,
    }
}

pub struct ExtractPaths<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> fmt::Debug for ExtractPaths<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPaths").finish()
    }
}

impl<T: FromSegments> Endpoint for ExtractPaths<T> {
    type Item = T;
    type Error = ExtractPathsError<T>;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        Some(T::from_segments(ctx.segments()).map_err(ExtractPathsError))
    }
}

pub struct ExtractPathsError<T: FromSegments = ()>(pub T::Err);

impl<T: FromSegments> fmt::Debug for ExtractPathsError<T>
where
    T::Err: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PathError").field(&self.0).finish()
    }
}

impl<T: FromSegments> fmt::Display for ExtractPathsError<T>
where
    T::Err: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: FromSegments> Error for ExtractPathsError<T>
where
    T::Err: Error,
{
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.0)
    }
}

impl<T: FromSegments> PartialEq for ExtractPathsError<T>
where
    T::Err: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HttpRequest;
    use test::EndpointTestExt;

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
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        assert_eq!(endpoint!("foo" => <_, ()>).run(request), Some(Ok(())),);
    }

    #[test]
    fn test_endpoint_reject_path() {
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        assert!(endpoint!("bar" => <_, ()>).run(request).is_none());
    }

    #[test]
    fn test_endpoint_match_multi_segments() {
        let request = HttpRequest::get("/foo/bar")
            .body(Default::default())
            .unwrap();
        assert_eq!(endpoint!("/foo/bar" => <_, ()>).run(request), Some(Ok(())));
    }

    #[test]
    fn test_endpoint_reject_multi_segments() {
        let request = HttpRequest::get("/foo/baz")
            .body(Default::default())
            .unwrap();
        assert!(endpoint!("/foo/bar" => <_, ()>).run(request).is_none());
    }

    #[test]
    fn test_endpoint_reject_short_path() {
        let request = HttpRequest::get("/foo/bar")
            .body(Default::default())
            .unwrap();
        assert!(endpoint!("/foo/bar/baz" => <_, ()>).run(request).is_none());
    }

    #[test]
    fn test_endpoint_match_all_path() {
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        assert_eq!(endpoint!("*" => <_, ()>).run(request), Some(Ok(())));
    }

    #[test]
    fn test_endpoint_extract_integer() {
        let request = HttpRequest::get("/42").body(Default::default()).unwrap();
        assert_eq!(path::<i32>().run(request), Some(Ok(42)));
    }

    #[test]
    fn test_endpoint_extract_wrong_integer() {
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        assert_eq!(path::<i32>().run(request).map(|r| r.is_err()), Some(true));
    }

    #[test]
    fn test_endpoint_extract_strings() {
        let request = HttpRequest::get("/foo/bar")
            .body(Default::default())
            .unwrap();
        assert_eq!(
            paths::<Vec<String>>().run(request),
            Some(Ok(vec!["foo".to_string(), "bar".to_string()]))
        );
    }
}
