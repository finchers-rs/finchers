//! Components for building endpoints which matches to a specific HTTP path.

pub mod encoded;
pub mod verb;

pub use {
    crate::path, //
    finchers_macros::ExtractPath,
};

use {
    self::encoded::FromEncodedStr,
    crate::{
        common::Tuple,
        endpoint::{
            Endpoint, //
            IsEndpoint,
            Oneshot,
            OneshotAction,
            PreflightContext,
        },
        error::Error,
    },
    http::StatusCode,
    percent_encoding::{
        percent_encode, //
        DEFAULT_ENCODE_SET,
    },
    std::{
        fmt, //
        marker::PhantomData,
        sync::Arc,
    },
};

// ==== ExtractPath ====

/// A macro for creating an endpoint that matches to the specific HTTP path.
#[macro_export]
macro_rules! path {
    ($path:expr) => {{
        #[derive($crate::endpoint::syntax::ExtractPath)]
        #[path = $path]
        struct __DerivedExtractPath(());

        $crate::endpoint::syntax::path::<__DerivedExtractPath>()
    }};

    (@$verb:ident $path:expr) => {
        $crate::endpoint::ext::EndpointExt::and(
            $crate::endpoint::syntax::verb::$verb(),
            $crate::endpoint::syntax::path!($path),
        )
    };
}

/// A trait that abstracts the extraction of values from HTTP path.
#[allow(missing_docs)]
pub trait ExtractPath {
    type Output: Tuple;

    fn extract(cx: &mut PreflightContext<'_>) -> Result<Self::Output, ExtractPathError>;
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct ExtractPathError(Error);

impl Into<Error> for ExtractPathError {
    fn into(self) -> Error {
        self.0
    }
}

impl ExtractPathError {
    #[allow(missing_docs)]
    pub fn new(cause: impl Into<Error>) -> Self {
        ExtractPathError(cause.into())
    }

    #[allow(missing_docs)]
    pub fn not_matched() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }
}

/// Creates an endpoint that matches to the specific HTTP path.
pub fn path<T>() -> Path<T>
where
    T: ExtractPath,
{
    Path {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Path<T> {
    _marker: PhantomData<T>,
}

mod path {
    use super::*;

    impl<T> IsEndpoint for Path<T> where T: ExtractPath {}

    impl<T, Bd> Endpoint<Bd> for Path<T>
    where
        T: ExtractPath,
    {
        type Output = T::Output;
        type Action = Oneshot<PathAction<T>>;

        fn action(&self) -> Self::Action {
            PathAction {
                _marker: PhantomData,
            }
            .into_action()
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct PathAction<T> {
        _marker: PhantomData<T>,
    }

    impl<T> OneshotAction for PathAction<T>
    where
        T: ExtractPath,
    {
        type Output = T::Output;

        #[inline]
        fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
            <T as ExtractPath>::extract(cx).map_err(Into::into)
        }
    }
}

// ==== MatchSegment =====

percent_encoding::define_encode_set! {
    /// The encode set for MatchSegment
    #[doc(hidden)]
    pub SEGMENT_ENCODE_SET = [DEFAULT_ENCODE_SET] | {'/'}
}

/// Create an endpoint which validates a path segment.
///
/// It takes a path segment from the context and check if it is equal
/// to the specified value.
pub fn segment(s: impl AsRef<str>) -> MatchSegment {
    let s = s.as_ref();
    debug_assert!(!s.is_empty());
    MatchSegment {
        encoded: Arc::new(percent_encode(s.as_bytes(), SEGMENT_ENCODE_SET).to_string()),
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchSegment {
    encoded: Arc<String>,
}

impl IsEndpoint for MatchSegment {}

impl<Bd> Endpoint<Bd> for MatchSegment {
    type Output = ();
    type Action = Oneshot<MatchSegmentAction>;

    fn action(&self) -> Self::Action {
        MatchSegmentAction {
            encoded: self.encoded.clone(),
        }
        .into_action()
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct MatchSegmentAction {
    encoded: Arc<String>,
}

impl OneshotAction for MatchSegmentAction {
    type Output = ();

    fn preflight(self, ecx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
        let s = ecx.next().ok_or_else(|| StatusCode::NOT_FOUND)?;
        if s == *self.encoded {
            Ok(())
        } else {
            Err(StatusCode::NOT_FOUND.into())
        }
    }
}

// ==== MatchEos ====

/// Create an endpoint which checks if the current context is reached the end of segments.
#[inline]
pub fn eos() -> MatchEos {
    MatchEos { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct MatchEos {
    _priv: (),
}

impl IsEndpoint for MatchEos {}

impl<Bd> Endpoint<Bd> for MatchEos {
    type Output = ();
    type Action = Oneshot<MatchEosAction>;

    fn action(&self) -> Self::Action {
        MatchEosAction { _priv: () }.into_action()
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct MatchEosAction {
    _priv: (),
}

impl OneshotAction for MatchEosAction {
    type Output = ();

    fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
        match cx.next() {
            None => Ok(()),
            Some(..) => Err(StatusCode::NOT_FOUND.into()),
        }
    }
}

// ==== Param ====

/// Create an endpoint which parses a path segment into the specified type.
///
/// This endpoint will skip the current request
/// if the segments is empty or the conversion is failed.
#[inline]
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

impl<T: FromEncodedStr> IsEndpoint for Param<T> {}

impl<T, Bd> Endpoint<Bd> for Param<T>
where
    T: FromEncodedStr,
{
    type Output = (T,);
    type Action = Oneshot<ParamAction<T>>;

    fn action(&self) -> Self::Action {
        ParamAction {
            _marker: PhantomData,
        }
        .into_action()
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct ParamAction<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> OneshotAction for ParamAction<T>
where
    T: FromEncodedStr,
{
    type Output = (T,);

    fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
        let s = cx.next().ok_or_else(|| StatusCode::NOT_FOUND)?;
        let x = T::from_encoded_str(s).map_err(Into::into)?;
        Ok((x,))
    }
}

// ==== Remains ====

/// Create an endpoint which parses the remaining path segments into the specified type.
///
/// This endpoint will skip the current request if the conversion is failed.
#[inline]
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

impl<T: FromEncodedStr> IsEndpoint for Remains<T> {}

impl<T, Bd> Endpoint<Bd> for Remains<T>
where
    T: FromEncodedStr,
{
    type Output = (T,);
    type Action = Oneshot<RemainsAction<T>>;

    fn action(&self) -> Self::Action {
        RemainsAction {
            _marker: PhantomData,
        }
        .into_action()
    }
}

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct RemainsAction<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> OneshotAction for RemainsAction<T>
where
    T: FromEncodedStr,
{
    type Output = (T,);

    fn preflight(self, cx: &mut PreflightContext<'_>) -> Result<Self::Output, Error> {
        let result = T::from_encoded_str(cx.remaining_path());
        let _ = cx.by_ref().count();
        result.map(|x| (x,)).map_err(Into::into)
    }
}
