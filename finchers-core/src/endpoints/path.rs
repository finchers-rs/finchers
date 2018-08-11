//! Components for parsing request path

use std::future::Future;
use std::marker::PhantomData;
use std::mem::PinMut;
use std::ops::Range;
use std::task::Poll;
use std::{error, fmt, task};

use futures_util::future;
use percent_encoding::{define_encode_set, percent_encode, DEFAULT_ENCODE_SET};

use crate::endpoint::EndpointBase;
use crate::error::Never;
use crate::generic::{one, One};
use crate::input::{with_get_cx, Cursor, FromSegment, Input, Segment};

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
/// # use finchers_core::endpoints::path::path;
/// # use finchers_core::local;
/// let endpoint = path("foo");
///
/// assert_eq!(local::get("/foo").apply(&endpoint), Some(Ok(())));
/// assert_eq!(local::get("/foo/bar").apply(&endpoint), Some(Ok(())));
/// assert_eq!(local::get("/bar").apply(&endpoint), None);
/// assert_eq!(local::get("/foobar").apply(&endpoint), None);
/// ```
///
/// Matches to multiple segments:
///
/// ```
/// # use finchers_core::endpoints::path::path;
/// # use finchers_core::local;
/// let endpoint = path("foo/bar");
///
/// assert_eq!(local::get("/foo/bar").apply(&endpoint), Some(Ok(())));
/// assert_eq!(local::get("/foo").apply(&endpoint), None);
/// assert_eq!(local::get("/foobar").apply(&endpoint), None);
/// ```
///
/// Matches to all remaining segments:
///
/// ```
/// # use finchers_core::endpoints::path::path;
/// # use finchers_core::endpoint::EndpointExt;
/// # use finchers_core::local;
/// let endpoint = path("foo").and(path("*"));
///
/// assert_eq!(local::get("/foo").apply(&endpoint), Some(Ok(())));
/// assert_eq!(local::get("/foo/").apply(&endpoint), Some(Ok(())));
/// assert_eq!(local::get("/foo/bar/baz").apply(&endpoint), Some(Ok(())));
/// assert_eq!(local::get("/bar").apply(&endpoint), None);
/// ```
pub fn path(s: &str) -> MatchPath {
    MatchPath::from_str(s).expect("The following path cannot be converted to an endpoint.")
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchPath {
    kind: MatchPathKind,
}

define_encode_set! {
    /// The encode set for MatchPath
    #[doc(hidden)]
    pub MATCH_PATH_ENCODE_SET = [DEFAULT_ENCODE_SET] | {'/'}
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
                let encoded = percent_encode(segment.as_bytes(), MATCH_PATH_ENCODE_SET).to_string();
                segments.push(encoded);
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

impl EndpointBase for MatchPath {
    type Ok = ();
    type Error = Never;
    type Future = future::Ready<Result<Self::Ok, Never>>;

    fn apply(&self, _: PinMut<Input>, mut cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        use self::MatchPathKind::*;
        match self.kind {
            Segments(ref segments) => {
                let mut matched = true;
                for segment in segments {
                    // FIXME: impl PartialEq for EncodedStr
                    unsafe {
                        matched = matched
                            && cursor.next_segment()?.as_encoded_str().as_bytes()
                                == segment.as_bytes();
                    }
                }
                if matched {
                    Some((future::ready(Ok(())), cursor))
                } else {
                    None
                }
            }
            AllSegments => {
                unsafe {
                    cursor.consume_all_segments();
                }
                Some((future::ready(Ok(())), cursor))
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
/// # #![feature(rust_2018_preview)]
/// # use finchers_core::endpoint::EndpointExt;
/// # use finchers_core::endpoints::path::{path, param};
/// let endpoint = path("posts").and(param())
///     .map_ok(|id: i32| (format!("id={}", id),));
/// ```
pub fn param<T>() -> Param<T>
where
    T: FromSegment,
{
    Param {
        _marker: PhantomData,
    }
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

impl<T> EndpointBase for Param<T>
where
    T: FromSegment,
{
    type Ok = One<T>;
    type Error = T::Error;
    type Future = ParamFuture<T>;

    fn apply(&self, _: PinMut<Input>, mut cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        let range = unsafe { cursor.next_segment()?.as_range() };
        Some((
            ParamFuture {
                range,
                _marker: PhantomData,
            },
            cursor,
        ))
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct ParamFuture<T> {
    range: Range<usize>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromSegment> Future for ParamFuture<T> {
    type Output = Result<One<T>, T::Error>;

    fn poll(self: PinMut<Self>, _: &mut task::Context) -> Poll<Self::Output> {
        Poll::Ready(with_get_cx(|input| {
            let s = Segment::new(input.request().uri().path(), self.range.clone());
            T::from_segment(s).map(one)
        }))
    }
}

/*
// ==== Params ====

/// Create an endpoint which extracts all remaining segments from
/// the path and converts them to the value of `T`.
///
/// If the conversion to `T` is failed, this endpoint will skip the request.
///
/// # Example
///
/// ```
/// #![feature(rust_2018_preview)]
/// # use finchers_core::ext::EndpointExt;
/// # use finchers_core::http::path::params;
/// # use std::path::PathBuf;
/// # fn main() {
/// let endpoint = params()
///     .map(|path: PathBuf| format!("path={}", path.display()));
/// # }
/// ```
pub fn params<T>() -> Params<T>
where
    T: FromSegments,
{
    Params {
        _marker: PhantomData,
    }
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

impl<T> EndpointBase for Params<T>
where
    T: FromSegments,
{
    type Ok = One<T>;
    type Error = Never;
    type Future = future::Ready<Result<Self::Ok, Self::Error>>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        T::from_segments(cx.segments())
            .map(one)
            .map(Ok)
            .map(future::ready)
            .ok()
    }
}
*/

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
            Ok(MatchPathKind::Segments(vec![
                "foo".to_owned(),
                "bar".to_owned(),
            ]))
        );
    }

    #[test]
    fn test_match_all_segments() {
        assert_eq!(
            MatchPath::from_str("*").map(|m| m.kind),
            Ok(MatchPathKind::AllSegments)
        );
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
}
