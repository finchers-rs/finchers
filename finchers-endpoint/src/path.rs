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

use finchers_core::error::{BadRequest, NotPresent};
use finchers_core::{Error, Input, Never};
use futures::future::{self, ok, FutureResult};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};
use {Context, Endpoint, IntoEndpoint};

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
        f.debug_struct("MatchPath").field("kind", &self.kind).finish()
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

    fn apply(&self, _: &Input, ctx: &mut Context) -> Option<Self::Future> {
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
    ExtractPath { _marker: PhantomData }
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

    fn apply(&self, _: &Input, ctx: &mut Context) -> Option<Self::Future> {
        ctx.segments().next().and_then(|s| T::from_segment(s).map(ok).ok())
    }
}

#[allow(missing_docs)]
pub fn path_req<T: FromSegment>() -> ExtractPathRequired<T> {
    ExtractPathRequired { _marker: PhantomData }
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

    fn apply(&self, _: &Input, ctx: &mut Context) -> Option<Self::Future> {
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
    ExtractPathOptional { _marker: PhantomData }
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

    fn apply(&self, _: &Input, ctx: &mut Context) -> Option<Self::Future> {
        Some(ok(ctx.segments().next().and_then(|s| T::from_segment(s).ok())))
    }
}

#[allow(missing_docs)]
pub fn paths<T: FromSegments>() -> ExtractPaths<T> {
    ExtractPaths { _marker: PhantomData }
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

    fn apply(&self, _: &Input, ctx: &mut Context) -> Option<Self::Future> {
        T::from_segments(ctx.segments()).map(ok).ok()
    }
}

#[allow(missing_docs)]
pub fn paths_req<T: FromSegments>() -> ExtractPathsRequired<T> {
    ExtractPathsRequired { _marker: PhantomData }
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

    fn apply(&self, _: &Input, ctx: &mut Context) -> Option<Self::Future> {
        Some(future::result(
            T::from_segments(ctx.segments())
                .map_err(BadRequest::new)
                .map_err(Into::into),
        ))
    }
}

#[allow(missing_docs)]
pub fn paths_opt<T: FromSegments>() -> ExtractPathsOptional<T> {
    ExtractPathsOptional { _marker: PhantomData }
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

    fn apply(&self, _: &Input, ctx: &mut Context) -> Option<Self::Future> {
        Some(ok(T::from_segments(ctx.segments()).ok()))
    }
}

/// An iterator of remaning path segments.
#[derive(Debug, Copy, Clone)]
pub struct Segments<'a> {
    path: &'a str,
    pos: usize,
    popped: usize,
}

impl<'a> From<&'a str> for Segments<'a> {
    fn from(path: &'a str) -> Self {
        debug_assert!(!path.is_empty());
        debug_assert_eq!(path.chars().next(), Some('/'));
        Segments {
            path,
            pos: 1,
            popped: 0,
        }
    }
}

impl<'a> Segments<'a> {
    /// Returns the remaining path in this segments
    #[inline]
    pub fn remaining_path(&self) -> &'a str {
        &self.path[self.pos..]
    }

    /// Returns the cursor position in the original path
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the number of segments already popped
    #[inline]
    pub fn popped(&self) -> usize {
        self.popped
    }
}

impl<'a> Iterator for Segments<'a> {
    type Item = Segment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.path.len() {
            return None;
        }
        if let Some(offset) = self.path[self.pos..].find('/') {
            let segment = Segment {
                s: &self.path[self.pos..self.pos + offset],
                start: self.pos,
                end: self.pos + offset,
            };
            self.pos += offset + 1;
            self.popped += 1;
            Some(segment)
        } else {
            let segment = Segment {
                s: &self.path[self.pos..],
                start: self.pos,
                end: self.path.len(),
            };
            self.pos = self.path.len();
            self.popped += 1;
            Some(segment)
        }
    }
}

/// A path segment in HTTP requests
#[derive(Debug, Copy, Clone)]
pub struct Segment<'a> {
    s: &'a str,
    start: usize,
    end: usize,
}

impl<'a> From<&'a str> for Segment<'a> {
    fn from(s: &'a str) -> Self {
        Segment {
            s,
            start: 0,
            end: s.len(),
        }
    }
}

impl<'a> Segment<'a> {
    /// Yields the underlying `str` slice.
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.s
    }

    /// Returns the start position of this segment in the original path
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end position of this segment in the original path
    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }
}

impl<'a> AsRef<[u8]> for Segment<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl<'a> AsRef<str> for Segment<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Deref for Segment<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

/// Represents the conversion from `Segment`
pub trait FromSegment: 'static + Sized {
    /// The error type returned from `from_segment`
    type Err: error::Error + Send + 'static;

    /// Create the instance of `Self` from a path segment
    fn from_segment(segment: Segment) -> Result<Self, Self::Err>;
}

macro_rules! impl_from_segment_from_str {
    ($($t:ty,)*) => {$(
        impl FromSegment for $t {
            type Err = <$t as FromStr>::Err;

            #[inline]
            fn from_segment(segment: Segment) -> Result<Self, Self::Err> {
                FromStr::from_str(&*segment)
            }
        }
    )*};
}

impl_from_segment_from_str! {
    String, bool, f32, f64,
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
    ::std::net::IpAddr,
    ::std::net::Ipv4Addr,
    ::std::net::Ipv6Addr,
    ::std::net::SocketAddr,
    ::std::net::SocketAddrV4,
    ::std::net::SocketAddrV6,
}

impl<T: FromSegment> FromSegment for Option<T> {
    type Err = Never;

    #[inline]
    fn from_segment(segment: Segment) -> Result<Self, Self::Err> {
        Ok(FromSegment::from_segment(segment).ok())
    }
}

impl<T: FromSegment> FromSegment for Result<T, T::Err> {
    type Err = Never;

    #[inline]
    fn from_segment(segment: Segment) -> Result<Self, Self::Err> {
        Ok(FromSegment::from_segment(segment))
    }
}

/// Represents the conversion from `Segments`
pub trait FromSegments: 'static + Sized {
    /// The error type from `from_segments`
    type Err: error::Error + Send + 'static;

    /// Create the instance of `Self` from the remaining path segments
    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err>;
}

impl<T: FromSegment> FromSegments for Vec<T> {
    type Err = T::Err;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        segments.into_iter().map(|s| T::from_segment(s)).collect()
    }
}

impl FromSegments for String {
    type Err = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        let s = segments.remaining_path().to_owned();
        let _ = segments.last();
        Ok(s)
    }
}

impl FromSegments for PathBuf {
    type Err = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        let s = PathBuf::from(segments.remaining_path());
        let _ = segments.last();
        Ok(s)
    }
}

impl<T: FromSegments> FromSegments for Option<T> {
    type Err = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        Ok(FromSegments::from_segments(segments).ok())
    }
}

impl<T: FromSegments> FromSegments for Result<T, T::Err> {
    type Err = Never;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Err> {
        Ok(FromSegments::from_segments(segments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_single_segment() {
        assert_eq!(match_("foo").map(|m| m.kind), Ok(Segments(vec!["foo".to_owned()])));
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
        assert_eq!(match_("").map(|m| m.kind), Err(ParseMatchError::EmptyString));
    }

    #[test]
    fn test_match_failure_empty_2() {
        assert_eq!(match_("foo//bar").map(|m| m.kind), Err(ParseMatchError::EmptyString));
    }

    #[test]
    fn test_segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        assert_eq!(segments.remaining_path(), "foo/bar.txt");
        assert_eq!(segments.next().map(|s| s.as_str()), Some("foo"));
        assert_eq!(segments.remaining_path(), "bar.txt");
        assert_eq!(segments.next().map(|s| s.as_str()), Some("bar.txt"));
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_str()), None);
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_str()), None);
    }

    #[test]
    fn test_segments_from_root_path() {
        let mut segments = Segments::from("/");
        assert_eq!(segments.remaining_path(), "");
        assert_eq!(segments.next().map(|s| s.as_str()), None);
    }

    #[test]
    fn test_from_segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        let result = FromSegments::from_segments(&mut segments);
        assert_eq!(result, Ok(PathBuf::from("foo/bar.txt")));
    }
}
