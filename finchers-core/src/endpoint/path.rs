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

use futures::future::{self, ok, FutureResult};
use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use errors::{BadRequest, Error, NotPresent};
use request::{FromSegment, FromSegments, Input};

#[allow(missing_docs)]
pub struct MatchPath {
    kind: MatchPathKind,
}

impl Clone for MatchPath {
    fn clone(&self) -> Self {
        MatchPath {
            kind: self.kind.clone(),
        }
    }
}

impl fmt::Debug for MatchPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MatchPath")
            .field("kind", &self.kind)
            .finish()
    }
}

impl MatchPath {
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

impl Endpoint for MatchPath {
    type Item = ();
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, _: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        match self.kind {
            Segments(ref segments) => {
                let mut matched = true;
                for segment in segments {
                    matched = matched && *ctx.segments().next()? == *segment;
                }
                if matched {
                    Some(ok(()))
                } else {
                    None
                }
            }
            AllSegments => {
                let _ = ctx.segments().count();
                Some(ok(()))
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
pub fn match_(s: &str) -> Result<MatchPath, ParseMatchError> {
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

    Ok(MatchPath { kind })
}

impl<'a> IntoEndpoint for &'a str {
    type Item = ();
    type Endpoint = MatchPath;

    fn into_endpoint(self) -> Self::Endpoint {
        match_(self).unwrap()
    }
}

impl IntoEndpoint for String {
    type Item = ();
    type Endpoint = MatchPath;

    fn into_endpoint(self) -> Self::Endpoint {
        match_(&self).unwrap()
    }
}

impl<'a> IntoEndpoint for Cow<'a, str> {
    type Item = ();
    type Endpoint = MatchPath;

    fn into_endpoint(self) -> Self::Endpoint {
        match_(&*self).unwrap()
    }
}

#[allow(missing_docs)]
pub fn path<T: FromSegment>() -> ExtractPath<T> {
    ExtractPath {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPath<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for ExtractPath<T> {}

impl<T> Clone for ExtractPath<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for ExtractPath<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPath").finish()
    }
}

impl<T: FromSegment> Endpoint for ExtractPath<T> {
    type Item = T;
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, _: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        ctx.segments()
            .next()
            .and_then(|s| T::from_segment(s).map(ok).ok())
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

impl<T: FromSegment> Endpoint for ExtractPathRequired<T> {
    type Item = T;
    type Future = FutureResult<T, Error>;

    fn apply(&self, _: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        let fut = match ctx.segments().next().map(|s| T::from_segment(s)) {
            Some(Ok(val)) => future::ok(val),
            Some(Err(e)) => future::err(BadRequest::new(e).into()),
            None => future::err(NotPresent::new("The number of path segments is insufficient").into()),
        };
        Some(fut)
    }
}

#[allow(missing_docs)]
pub fn path_opt<T: FromSegment>() -> ExtractPathOptional<T> {
    ExtractPathOptional {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPathOptional<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for ExtractPathOptional<T> {}

impl<T> Clone for ExtractPathOptional<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for ExtractPathOptional<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPathOptional").finish()
    }
}

impl<T: FromSegment> Endpoint for ExtractPathOptional<T> {
    type Item = Option<T>;
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, _: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        Some(ok(ctx.segments()
            .next()
            .and_then(|s| T::from_segment(s).ok())))
    }
}

#[allow(missing_docs)]
pub fn paths<T: FromSegments>() -> ExtractPaths<T> {
    ExtractPaths {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPaths<T> {
    _marker: PhantomData<fn() -> (T)>,
}

impl<T> Copy for ExtractPaths<T> {}

impl<T> Clone for ExtractPaths<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for ExtractPaths<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPaths").finish()
    }
}

impl<T: FromSegments> Endpoint for ExtractPaths<T> {
    type Item = T;
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, _: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        T::from_segments(ctx.segments()).map(ok).ok()
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

impl<T: FromSegments> Endpoint for ExtractPathsRequired<T> {
    type Item = T;
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, _: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        Some(future::result(
            T::from_segments(ctx.segments())
                .map_err(BadRequest::new)
                .map_err(Into::into),
        ))
    }
}

#[allow(missing_docs)]
pub fn paths_opt<T: FromSegments>() -> ExtractPathsOptional<T> {
    ExtractPathsOptional {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct ExtractPathsOptional<T> {
    _marker: PhantomData<fn() -> (T)>,
}

impl<T> Copy for ExtractPathsOptional<T> {}

impl<T> Clone for ExtractPathsOptional<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for ExtractPathsOptional<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtractPathsOptional").finish()
    }
}

impl<T: FromSegments> Endpoint for ExtractPathsOptional<T> {
    type Item = Option<T>;
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, _: &Input, ctx: &mut EndpointContext) -> Option<Self::Future> {
        Some(ok(T::from_segments(ctx.segments()).ok()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use endpoint::endpoint;
    use http::Request;
    use test::EndpointTestExt;

    #[test]
    fn test_match_single_segment() {
        assert_eq!(
            match_("foo").map(|m| m.kind),
            Ok(Segments(vec!["foo".to_owned()]))
        );
    }

    #[test]
    fn test_match_multi_segments() {
        assert_eq!(
            match_("foo/bar").map(|m| m.kind),
            Ok(Segments(vec!["foo".to_owned(), "bar".to_owned()]))
        );
    }

    #[test]
    fn test_match_all_segments() {
        assert_eq!(match_("*").map(|m| m.kind), Ok(AllSegments));
    }

    #[test]
    fn test_match_failure_empty() {
        assert_eq!(
            match_("").map(|m| m.kind),
            Err(ParseMatchError::EmptyString)
        );
    }

    #[test]
    fn test_match_failure_empty_2() {
        assert_eq!(
            match_("foo//bar").map(|m| m.kind),
            Err(ParseMatchError::EmptyString)
        );
    }

    #[test]
    fn test_endpoint_match_path() {
        let request = Request::get("/foo").body(()).unwrap();
        assert_eq!(endpoint("foo").run(request).ok(), Some(()));
    }

    #[test]
    fn test_endpoint_reject_path() {
        let request = Request::get("/foo").body(()).unwrap();
        assert!(endpoint("bar").run(request).is_noroute());
    }

    #[test]
    fn test_endpoint_match_multi_segments() {
        let request = Request::get("/foo/bar").body(()).unwrap();
        assert_eq!(endpoint("/foo/bar").run(request).ok(), Some(()));
    }

    #[test]
    fn test_endpoint_reject_multi_segments() {
        let request = Request::get("/foo/baz").body(()).unwrap();
        assert!(endpoint("/foo/bar").run(request).is_noroute());
    }

    #[test]
    fn test_endpoint_reject_short_path() {
        let request = Request::get("/foo/bar").body(()).unwrap();
        assert!(endpoint("/foo/bar/baz").run(request).is_noroute());
    }

    #[test]
    fn test_endpoint_match_all_path() {
        let request = Request::get("/foo").body(()).unwrap();
        assert_eq!(endpoint("*").run(request).ok(), Some(()));
    }

    #[test]
    fn test_endpoint_extract_integer() {
        let request = Request::get("/42").body(()).unwrap();
        assert_eq!(path().run(request).ok(), Some(42i32));
    }

    #[test]
    fn test_endpoint_extract_wrong_integer() {
        let request = Request::get("/foo").body(()).unwrap();
        assert!(path::<i32>().run(request).is_noroute());
    }

    #[test]
    fn test_endpoint_extract_wrong_integer_result() {
        let request = Request::get("/foo").body(()).unwrap();
        match path::<Result<i32, _>>().run(request).ok() {
            Some(Err(..)) => (),
            _ => panic!("assertion failed"),
        }
    }

    #[test]
    fn test_endpoint_extract_wrong_integer_required() {
        let request = Request::get("/foo").body(()).unwrap();
        assert!(path_req::<i32>().run(request).is_err());
    }

    #[test]
    fn test_endpoint_extract_strings() {
        let request = Request::get("/foo/bar").body(()).unwrap();
        assert_eq!(
            paths::<Vec<String>>().run(request).ok(),
            Some(vec!["foo".to_string(), "bar".to_string()])
        );
    }
}
