//! Components for parsing request path

use std::fmt;
use std::marker::PhantomData;
use std::mem::PinMut;

use failure::Fail;
use futures_util::future;
use http::StatusCode;
use percent_encoding::{define_encode_set, percent_encode, DEFAULT_ENCODE_SET};

use crate::endpoint::Endpoint;
use crate::error::{Error, HttpError};
use crate::generic::{one, One};
use crate::input::{Cursor, FromEncodedStr, Input};

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

impl Endpoint for MatchPath {
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(
        &self,
        _: PinMut<'_, Input>,
        mut cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        if cursor.next()? == self.encoded {
            Some((future::ready(Ok(())), cursor))
        } else {
            None
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

impl Endpoint for EndPath {
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(
        &self,
        _: PinMut<'_, Input>,
        mut cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        if cursor.next().is_none() {
            Some((future::ready(Ok(())), cursor))
        } else {
            None
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
    T::Error: Fail,
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

impl<T> Endpoint for Param<T>
where
    T: FromEncodedStr,
    T::Error: Fail,
{
    type Output = One<T>;
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(
        &self,
        _: PinMut<'_, Input>,
        mut cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        let result = T::from_encoded_str(cursor.next()?)
            .map(one)
            .map_err(|cause| ParamError { cause }.into());
        Some((future::ready(result), cursor))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Fail)]
#[fail(display = "failed to parse a path segment: {}", cause)]
pub struct ParamError<E: Fail> {
    cause: E,
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
    T::Error: Fail,
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

impl<T> Endpoint for Remains<T>
where
    T: FromEncodedStr,
    T::Error: Fail,
{
    type Output = One<T>;
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(
        &self,
        _: PinMut<'_, Input>,
        mut cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        let result = T::from_encoded_str(cursor.remaining_path())
            .map(one)
            .map_err(|cause| ParamError { cause }.into());
        let _ = cursor.by_ref().count();
        Some((future::ready(result), cursor))
    }
}
