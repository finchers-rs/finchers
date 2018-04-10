//! Components for parsing request path

use finchers_core::endpoint::{Context, Endpoint, Error, Segment, Segments};
use futures::future::{ok, FutureResult};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::str::FromStr;
use std::{error, fmt};

// ==== MatchPath =====

/// Create an endpoint which takes some segments from the path
/// and check if the segments are matched to the certain pattern.
///
/// # Panics
/// This function will be panic if the given argument is an invalid
/// pattern.
///
/// # Example
///
/// Matches to a single segment:
///
/// ```
/// # extern crate finchers_http;
/// # extern crate finchers_endpoint;
/// # use finchers_http::path::path;
/// # use finchers_endpoint::{ok, EndpointExt};
/// # fn main() {
/// let endpoint = path("foo").and(ok("matched"));
/// # }
/// ```
///
/// Matches to multiple segments:
///
/// ```
/// # extern crate finchers_http;
/// # extern crate finchers_endpoint;
/// # use finchers_http::path::path;
/// # use finchers_endpoint::{ok, EndpointExt};
/// # fn main() {
/// let endpoint = path("foo/bar").and(ok("matched"));
/// # }
/// ```
///
/// Matches to all remaining segments:
///
/// ```
/// # extern crate finchers_http;
/// # extern crate finchers_endpoint;
/// # use finchers_http::path::path;
/// # use finchers_endpoint::{ok, EndpointExt};
/// # fn main() {
/// let endpoint = path("*").and(ok("matched"));
/// # }
/// ```
pub fn path(s: &str) -> MatchPath {
    MatchPath::from_str(s).expect("The following path cannot be converted to an endpoint.")
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchPath {
    kind: MatchPathKind,
}

impl MatchPath {
    /// Create an instance of `MatchPath` from given string.
    pub fn from_str(s: &str) -> Result<MatchPath, ParseMatchError> {
        use self::MatchPathKind::*;
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

    /// Return the kind of this endpoint.
    pub fn kind(&self) -> &MatchPathKind {
        &self.kind
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum MatchPathKind {
    /// Matched to (multiple) path segments.
    Segments(Vec<String>),
    /// Matched to all remaining path segments.
    AllSegments,
}

impl Endpoint for MatchPath {
    type Item = ();
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        use self::MatchPathKind::*;
        match self.kind {
            Segments(ref segments) => {
                let mut matched = true;
                for segment in segments {
                    matched = matched && *cx.segments().next()? == *segment;
                }
                if matched {
                    Some(ok(()))
                } else {
                    None
                }
            }
            AllSegments => {
                let _ = cx.segments().count();
                Some(ok(()))
            }
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ParseMatchError {
    EmptyString,
}

impl fmt::Display for ParseMatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseMatchError::EmptyString => f.write_str("empty str"),
        }
    }
}

impl error::Error for ParseMatchError {
    fn description(&self) -> &str {
        match *self {
            ParseMatchError::EmptyString => "empty string",
        }
    }
}

// ==== Param ====

/// Create an endpoint which extracts one segment from the path
/// and converts it to the value of `T`.
///
/// If the segments is empty of the conversion to `T` is failed,
/// this endpoint will skip the request.
///
/// # Example
///
/// ```
/// # extern crate finchers_http;
/// # extern crate finchers_endpoint;
/// # use finchers_endpoint::EndpointExt;
/// # use finchers_http::path::param;
/// # fn main() {
/// let endpoint = param()
///     .map(|id: i32| format!("id={}", id));
/// # }
/// ```
///
/// Custom handling for the conversion error:
///
/// ```
/// # extern crate finchers_core;
/// # extern crate finchers_endpoint;
/// # extern crate finchers_http;
/// # use finchers_core::error::BadRequest;
/// # use finchers_endpoint::EndpointExt;
/// # use finchers_http::path::{param, FromSegment};
/// # fn main() {
/// let endpoint = param()
///     .try_abort(|id: Result<i32, _>| id.map_err(BadRequest::new));
/// # }
/// ```
pub fn param<T>() -> Param<T>
where
    T: FromSegment + Send,
{
    Param { _marker: PhantomData }
}

#[allow(missing_docs)]
pub struct Param<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Param<T> {}

impl<T> Clone for Param<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Param<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Param").finish()
    }
}

impl<T> Endpoint for Param<T>
where
    T: FromSegment + Send,
{
    type Item = T;
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let s = cx.segments().next()?;
        T::from_segment(s).map(ok).ok()
    }
}

/// Trait representing the conversion from "Segment".
pub trait FromSegment: 'static + Sized {
    /// The error type returned from "from_segment".
    type Error;

    /// Perform conversion from "Segment" to "Self".
    fn from_segment(segment: Segment) -> Result<Self, Self::Error>;
}

macro_rules! impl_from_segment_from_str {
    ($($t:ty,)*) => {$(
        impl FromSegment for $t {
            type Error = <$t as FromStr>::Err;

            #[inline]
            fn from_segment(segment: Segment) -> Result<Self, Self::Error> {
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
    type Error = !;

    fn from_segment(segment: Segment) -> Result<Self, Self::Error> {
        Ok(T::from_segment(segment).ok())
    }
}

impl<T: FromSegment> FromSegment for Result<T, T::Error> {
    type Error = !;

    fn from_segment(segment: Segment) -> Result<Self, Self::Error> {
        Ok(T::from_segment(segment))
    }
}

// ==== Params ====

/// Create an endpoint which extracts all remaining segments from
/// the path and converts them to the value of `T`.
///
/// If the conversion to `T` is failed, this endpoint will skip the request.
///
/// # Example
///
/// ```
/// # extern crate finchers_endpoint;
/// # extern crate finchers_http;
/// # use finchers_endpoint::EndpointExt;
/// # use finchers_http::path::params;
/// # use std::path::PathBuf;
/// # fn main() {
/// let endpoint = params()
///     .map(|path: PathBuf| format!("path={}", path.display()));
/// # }
/// ```
pub fn params<T>() -> Params<T>
where
    T: FromSegments + Send,
{
    Params { _marker: PhantomData }
}

#[allow(missing_docs)]
pub struct Params<T> {
    _marker: PhantomData<fn() -> (T)>,
}

impl<T> Copy for Params<T> {}

impl<T> Clone for Params<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Params<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Params").finish()
    }
}

impl<T> Endpoint for Params<T>
where
    T: FromSegments + Send,
{
    type Item = T;
    type Future = FutureResult<Self::Item, Error>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        T::from_segments(cx.segments()).map(ok).ok()
    }
}

/// Trait representing the conversion from a `Segments`
pub trait FromSegments: 'static + Sized {
    /// The error type returned from `from_segments`
    type Error;

    /// Perform conversion from `Segments` to `Self`.
    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error>;
}

impl<T: FromSegment> FromSegments for Vec<T> {
    type Error = T::Error;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        segments.into_iter().map(|s| T::from_segment(s)).collect()
    }
}

impl FromSegments for String {
    type Error = !;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        let s = segments.remaining_path().to_owned();
        let _ = segments.last();
        Ok(s)
    }
}

impl FromSegments for PathBuf {
    type Error = !;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        let s = PathBuf::from(segments.remaining_path());
        let _ = segments.last();
        Ok(s)
    }
}

impl<T: FromSegments> FromSegments for Option<T> {
    type Error = !;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        Ok(T::from_segments(segments).ok())
    }
}

impl<T: FromSegments> FromSegments for Result<T, T::Error> {
    type Error = !;

    fn from_segments(segments: &mut Segments) -> Result<Self, Self::Error> {
        Ok(T::from_segments(segments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_single_segment() {
        assert_eq!(
            MatchPath::from_str("foo").map(|m| m.kind),
            Ok(MatchPathKind::Segments(vec!["foo".to_owned()]))
        );
    }

    #[test]
    fn test_match_multi_segments() {
        assert_eq!(
            MatchPath::from_str("foo/bar").map(|m| m.kind),
            Ok(MatchPathKind::Segments(vec!["foo".to_owned(), "bar".to_owned()]))
        );
    }

    #[test]
    fn test_match_all_segments() {
        assert_eq!(MatchPath::from_str("*").map(|m| m.kind), Ok(MatchPathKind::AllSegments));
    }

    #[test]
    fn test_match_failure_empty() {
        assert_eq!(
            MatchPath::from_str("").map(|m| m.kind),
            Err(ParseMatchError::EmptyString)
        );
    }

    #[test]
    fn test_match_failure_empty_2() {
        assert_eq!(
            MatchPath::from_str("foo//bar").map(|m| m.kind),
            Err(ParseMatchError::EmptyString)
        );
    }

    #[test]
    fn test_from_segments() {
        let mut segments = Segments::from("/foo/bar.txt");
        let result = FromSegments::from_segments(&mut segments);
        assert_eq!(result, Ok(PathBuf::from("foo/bar.txt")));
    }
}
