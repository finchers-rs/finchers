//! Components for parsing request path

use std::fmt;
use std::marker::PhantomData;

use failure::Fail;
use futures_util::future;
use http::StatusCode;
use percent_encoding::{define_encode_set, percent_encode, DEFAULT_ENCODE_SET};

use crate::endpoint::{Context, Endpoint, EndpointErrorKind, EndpointResult};
use crate::error::{Error, HttpError};
use crate::generic::{one, One};
use crate::input::FromEncodedStr;

define_encode_set! {
    /// The encode set for MatchPath
    #[doc(hidden)]
    pub MATCH_PATH_ENCODE_SET = [DEFAULT_ENCODE_SET] | {'/'}
}

// ==== MatchPath =====

/// Create an endpoint which takes a path segment and check if it equals
/// to the specified value.
pub fn path(s: impl AsRef<str>) -> MatchPath {
    let s = s.as_ref();
    debug_assert!(!s.is_empty());
    MatchPath {
        encoded: percent_encode(s.as_bytes(), MATCH_PATH_ENCODE_SET).to_string(),
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchPath {
    encoded: String,
}

impl<'a> Endpoint<'a> for MatchPath {
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let s = ecx
            .next_segment()
            .ok_or_else(|| EndpointErrorKind::NotMatched)?;
        if s == self.encoded {
            Ok(future::ready(Ok(())))
        } else {
            Err(EndpointErrorKind::NotMatched)
        }
    }
}

// ==== EndPath ====

/// Create an endpoint to check if the path has reached the end.
pub fn end() -> EndPath {
    EndPath { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct EndPath {
    _priv: (),
}

impl<'a> Endpoint<'a> for EndPath {
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match ecx.next_segment() {
            None => Ok(future::ready(Ok(()))),
            Some(..) => Err(EndpointErrorKind::NotMatched),
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
/// # use finchers::endpoint::EndpointExt;
/// # use finchers::endpoints::path::{path, param};
/// let endpoint = path("posts").and(param())
///     .map(|id: i32| (format!("id={}", id),));
/// ```
pub fn param<T>() -> Param<T>
where
    T: FromEncodedStr,
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Param").finish()
    }
}

impl<'a, T> Endpoint<'a> for Param<T>
where
    T: FromEncodedStr,
{
    type Output = One<T>;
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let s = ecx
            .next_segment()
            .ok_or_else(|| EndpointErrorKind::NotMatched)?;
        let result = T::from_encoded_str(s)
            .map(one)
            .map_err(|cause| ParamError { cause }.into());
        Ok(future::ready(result))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ParamError<E> {
    cause: E,
}

impl<E: fmt::Display> fmt::Display for ParamError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse a path segment: {}", self.cause)
    }
}

impl<E: Fail> HttpError for ParamError<E> {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

// ==== Remains ====

/// Create an endpoint which extracts all remaining segments from
/// the path and converts them to the value of `T`.
///
/// If the conversion to `T` is failed, this endpoint will skip the request.
///
/// # Example
///
/// ```
/// #![feature(rust_2018_preview)]
/// # use finchers::endpoint::EndpointExt;
/// # use finchers::endpoints::path::{path, remains};
/// # use std::path::PathBuf;
/// let endpoint = path("foo").and(remains())
///     .map(|path: PathBuf| format!("path={}", path.display()));
/// ```
pub fn remains<T>() -> Remains<T>
where
    T: FromEncodedStr,
{
    Remains {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Remains<T> {
    _marker: PhantomData<fn() -> (T)>,
}

impl<T> Copy for Remains<T> {}

impl<T> Clone for Remains<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Remains<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Remains").finish()
    }
}

impl<'a, T> Endpoint<'a> for Remains<T>
where
    T: FromEncodedStr,
{
    type Output = One<T>;
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let result = T::from_encoded_str(ecx.remaining_path())
            .map(one)
            .map_err(|cause| ParamError { cause }.into());
        while let Some(..) = ecx.next_segment() {}
        Ok(future::ready(result))
    }
}
