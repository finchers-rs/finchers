//! Components for parsing request path
//!
//! Provided endpoints are as follows:
//!
//! * `MatchPath` - Checks if the prefix of remaining path(s) are matched to certain segments
//! * `ExtractPath<T>` - Takes a path segment and converts into a value of `T`
//! * `ExtractPaths<T>` - Convert the remaining path segments into the value of `T`
//!
//! By default, the endpoint `ExtractPath<T>` does not match to the input if the given path segment cannot be converted to `T`.
//! If you would like to change the behaviour, use `ExtractPath<Result<T, T::Err>>` or `ExtractPathRequired<T>` as follows:
//!
//! ```ignore
//! path::<Result<i32, _>, _>().and_then(|r| r)
//!     .assert_types::<i32, <i32 as FromSegments>::Err>()
//!
//! // or
//! path_req::<i32>()
//!     .assert_types::<i32, ExtractPathError<i32>>()
//! ```

use std::borrow::Cow;
use std::fmt;
use std::error::Error;
use std::marker::PhantomData;
use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use http::{FromSegment, FromSegments};
use errors::HttpError;

#[allow(missing_docs)]
pub struct MatchPath<E> {
    kind: MatchPathKind,
    _marker: PhantomData<fn() -> E>,
}

impl<E> Clone for MatchPath<E> {
    fn clone(&self) -> Self {
        MatchPath {
            kind: self.kind.clone(),
            _marker: PhantomData,
        }
    }
}

impl<E> fmt::Debug for MatchPath<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MatchPath")
            .field("kind", &self.kind)
            .finish()
    }
}

impl<E> MatchPath<E> {
    #[allow(missing_docs)]
    pub fn kind(&self) -> &MatchPathKind {
        &self.kind
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum MatchPathKind {
    Segments(Vec<String>),
    AllSegments,
}
use self::MatchPathKind::*;

impl<E: HttpError> Endpoint for MatchPath<E> {
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

#[allow(missing_docs)]
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

impl<'a, E: HttpError> IntoEndpoint<(), E> for &'a str {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(self).unwrap()
    }
}

impl<E: HttpError> IntoEndpoint<(), E> for String {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(&self).unwrap()
    }
}

impl<'a, E: HttpError> IntoEndpoint<(), E> for Cow<'a, str> {
    type Endpoint = MatchPath<E>;
    fn into_endpoint(self) -> Self::Endpoint {
        match_(&*self).unwrap()
    }
}

#[allow(missing_docs)]
pub fn path<T: FromSegment, E>() -> ExtractPath<T, E> {
    ExtractPath {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPath<T, E> {
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Copy for ExtractPath<T, E> {}

impl<T, E> Clone for ExtractPath<T, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> fmt::Debug for ExtractPath<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPath").finish()
    }
}

impl<T: FromSegment, E: HttpError> Endpoint for ExtractPath<T, E> {
    type Item = T;
    type Error = E;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        ctx.segments()
            .next()
            .and_then(|s| T::from_segment(&s).map(Ok).ok())
    }
}

#[allow(missing_docs)]
pub fn path_req<T: FromSegment>() -> ExtractPathRequired<T> {
    ExtractPathRequired {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPathRequired<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for ExtractPathRequired<T> {}

impl<T> Clone for ExtractPathRequired<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for ExtractPathRequired<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPathRequired").finish()
    }
}

impl<T: FromSegment> Endpoint for ExtractPathRequired<T>
where
    T::Err: Error,
{
    type Item = T;
    type Error = ExtractPathError<T>;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        ctx.segments()
            .next()
            .map(|s| T::from_segment(&s).map_err(ExtractPathError))
    }
}

#[allow(missing_docs)]
pub struct ExtractPathError<T: FromSegment>(pub T::Err);

impl<T: FromSegment> fmt::Debug for ExtractPathError<T>
where
    T::Err: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PathError").field(&self.0).finish()
    }
}

impl<T: FromSegment> fmt::Display for ExtractPathError<T>
where
    T::Err: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: FromSegment> Error for ExtractPathError<T>
where
    T::Err: Error,
{
    #[inline]
    fn description(&self) -> &str {
        self.0.description()
    }

    #[inline]
    fn cause(&self) -> Option<&Error> {
        self.0.cause()
    }
}

impl<T: FromSegment> PartialEq for ExtractPathError<T>
where
    T::Err: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }
}

#[allow(missing_docs)]
pub fn path_opt<T: FromSegment, E>() -> ExtractPathOptional<T, E> {
    ExtractPathOptional {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPathOptional<T, E> {
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Copy for ExtractPathOptional<T, E> {}

impl<T, E> Clone for ExtractPathOptional<T, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> fmt::Debug for ExtractPathOptional<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPathOptional").finish()
    }
}

impl<T: FromSegment, E: HttpError> Endpoint for ExtractPathOptional<T, E> {
    type Item = Option<T>;
    type Error = E;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        ctx.segments().next().map(|s| Ok(T::from_segment(&s).ok()))
    }
}

#[allow(missing_docs)]
pub fn paths<T: FromSegments, E>() -> ExtractPaths<T, E> {
    ExtractPaths {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPaths<T, E> {
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Copy for ExtractPaths<T, E> {}

impl<T, E> Clone for ExtractPaths<T, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> fmt::Debug for ExtractPaths<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPaths").finish()
    }
}

impl<T: FromSegments, E: HttpError> Endpoint for ExtractPaths<T, E> {
    type Item = T;
    type Error = E;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        T::from_segments(ctx.segments()).map(Ok).ok()
    }
}

#[allow(missing_docs)]
pub fn paths_req<T: FromSegments>() -> ExtractPathsRequired<T> {
    ExtractPathsRequired {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPathsRequired<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for ExtractPathsRequired<T> {}

impl<T> Clone for ExtractPathsRequired<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for ExtractPathsRequired<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPathsRequired").finish()
    }
}

impl<T: FromSegments> Endpoint for ExtractPathsRequired<T>
where
    T::Err: Error,
{
    type Item = T;
    type Error = ExtractPathsError<T>;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        Some(T::from_segments(ctx.segments()).map_err(ExtractPathsError))
    }
}

#[allow(missing_docs)]
pub struct ExtractPathsError<T: FromSegments>(pub T::Err);

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

#[allow(missing_docs)]
pub fn paths_opt<T: FromSegments, E>() -> ExtractPathsOptional<T, E> {
    ExtractPathsOptional {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPathsOptional<T, E> {
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> Copy for ExtractPathsOptional<T, E> {}

impl<T, E> Clone for ExtractPathsOptional<T, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> fmt::Debug for ExtractPathsOptional<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPathsOptional").finish()
    }
}

impl<T: FromSegments, E: HttpError> Endpoint for ExtractPathsOptional<T, E> {
    type Item = Option<T>;
    type Error = E;
    type Result = Result<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        Some(Ok(T::from_segments(ctx.segments()).ok()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use endpoint::{endpoint, Endpoint};
    use errors::NeverReturn;
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
        assert_eq!(
            endpoint("foo")
                .assert_types::<_, NeverReturn>()
                .run(request),
            Some(Ok(())),
        );
    }

    #[test]
    fn test_endpoint_reject_path() {
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        assert!(
            endpoint("bar")
                .assert_types::<_, NeverReturn>()
                .run(request)
                .is_none()
        );
    }

    #[test]
    fn test_endpoint_match_multi_segments() {
        let request = HttpRequest::get("/foo/bar")
            .body(Default::default())
            .unwrap();
        assert_eq!(
            endpoint("/foo/bar")
                .assert_types::<_, NeverReturn>()
                .run(request),
            Some(Ok(()))
        );
    }

    #[test]
    fn test_endpoint_reject_multi_segments() {
        let request = HttpRequest::get("/foo/baz")
            .body(Default::default())
            .unwrap();
        assert!(
            endpoint("/foo/bar")
                .assert_types::<_, NeverReturn>()
                .run(request)
                .is_none()
        );
    }

    #[test]
    fn test_endpoint_reject_short_path() {
        let request = HttpRequest::get("/foo/bar")
            .body(Default::default())
            .unwrap();
        assert!(
            endpoint("/foo/bar/baz")
                .assert_types::<_, NeverReturn>()
                .run(request)
                .is_none()
        );
    }

    #[test]
    fn test_endpoint_match_all_path() {
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        assert_eq!(
            endpoint("*").assert_types::<_, NeverReturn>().run(request),
            Some(Ok(()))
        );
    }

    #[test]
    fn test_endpoint_extract_integer() {
        let request = HttpRequest::get("/42").body(Default::default()).unwrap();
        assert_eq!(path::<i32, NeverReturn>().run(request), Some(Ok(42)));
    }

    #[test]
    fn test_endpoint_extract_wrong_integer() {
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        assert_eq!(path::<i32, NeverReturn>().run(request), None);
    }

    #[test]
    fn test_endpoint_extract_wrong_integer_result() {
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        match path::<Result<i32, _>, NeverReturn>().run(request) {
            Some(Ok(Err(..))) => (),
            _ => panic!("assertion failed"),
        }
    }

    #[test]
    fn test_endpoint_extract_wrong_integer_required() {
        let request = HttpRequest::get("/foo").body(Default::default()).unwrap();
        assert_eq!(
            path_req::<i32>().run(request).map(|r| r.is_err()),
            Some(true)
        );
    }

    #[test]
    fn test_endpoint_extract_strings() {
        let request = HttpRequest::get("/foo/bar")
            .body(Default::default())
            .unwrap();
        assert_eq!(
            paths::<Vec<String>, NeverReturn>().run(request),
            Some(Ok(vec!["foo".to_string(), "bar".to_string()]))
        );
    }
}
