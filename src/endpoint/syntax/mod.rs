//! Components for building endpoints which matches to a specific HTTP path.

#[allow(missing_docs)]
pub mod verb;

use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

use percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};

use endpoint::{ApplyContext, Endpoint, EndpointError, EndpointResult, IntoEndpoint};
use error;
use error::Error;
use input::FromEncodedStr;

#[doc(hidden)]
#[derive(Debug)]
#[must_use = "futures does not anything unless polled."]
pub struct Matched {
    _priv: (),
}

impl ::futures::Future for Matched {
    type Item = ();
    type Error = Error;

    #[inline]
    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        Ok(().into())
    }
}

#[doc(hidden)]
#[derive(Debug)]
#[must_use = "futures does not anything unless polled."]
pub struct Extracted<T>(Option<T>);

impl<T> ::futures::Future for Extracted<T> {
    type Item = (T,);
    type Error = Error;

    #[inline]
    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        let x = self.0.take().expect("This future has already polled");
        Ok((x,).into())
    }
}

// ==== MatchSegment =====

define_encode_set! {
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
    (MatchSegment {
        encoded: percent_encode(s.as_bytes(), SEGMENT_ENCODE_SET).to_string(),
    }).with_output::<()>()
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchSegment {
    encoded: String,
}

impl<'a> Endpoint<'a> for MatchSegment {
    type Output = ();
    type Future = Matched;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> EndpointResult<Self::Future> {
        let s = ecx.next_segment().ok_or_else(EndpointError::not_matched)?;
        if s == self.encoded {
            Ok(Matched { _priv: () })
        } else {
            Err(EndpointError::not_matched())
        }
    }
}

impl<'a, 's> IntoEndpoint<'a> for &'s str {
    type Output = ();
    type Endpoint = MatchSegment;

    #[inline(always)]
    fn into_endpoint(self) -> Self::Endpoint {
        segment(self)
    }
}

impl<'a> IntoEndpoint<'a> for String {
    type Output = ();
    type Endpoint = MatchSegment;

    #[inline(always)]
    fn into_endpoint(self) -> Self::Endpoint {
        segment(self)
    }
}

impl<'a, 's> IntoEndpoint<'a> for Cow<'s, str> {
    type Output = ();
    type Endpoint = MatchSegment;

    #[inline(always)]
    fn into_endpoint(self) -> Self::Endpoint {
        segment(self)
    }
}

// ==== MatchEos ====

/// Create an endpoint which checks if the current context is reached the end of segments.
#[inline]
pub fn eos() -> MatchEos {
    (MatchEos { _priv: () }).with_output::<()>()
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct MatchEos {
    _priv: (),
}

impl<'a> Endpoint<'a> for MatchEos {
    type Output = ();
    type Future = Matched;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> EndpointResult<Self::Future> {
        match ecx.next_segment() {
            None => Ok(Matched { _priv: () }),
            Some(..) => Err(EndpointError::not_matched()),
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
    (Param {
        _marker: PhantomData,
    }).with_output::<(T,)>()
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
    type Output = (T,);
    type Future = Extracted<T>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> EndpointResult<Self::Future> {
        let s = ecx.next_segment().ok_or_else(EndpointError::not_matched)?;
        let x =
            T::from_encoded_str(s).map_err(|err| EndpointError::custom(error::bad_request(err)))?;
        Ok(Extracted(Some(x)))
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
    (Remains {
        _marker: PhantomData,
    }).with_output::<(T,)>()
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
    type Output = (T,);
    type Future = Extracted<T>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> EndpointResult<Self::Future> {
        let result = T::from_encoded_str(ecx.remaining_path())
            .map_err(|err| EndpointError::custom(error::bad_request(err)));
        while let Some(..) = ecx.next_segment() {}
        result.map(|x| Extracted(Some(x)))
    }
}

/// A helper macro for creating an endpoint which matches to the specified HTTP path.
///
/// # Example
///
/// The following macro call
///
/// ```ignore
/// path!(@get / "api" / "v1" / "posts" / i32);
/// ```
///
/// will be roughly expanded to:
///
/// ```ignore
/// syntax::verb::get()
///     .and("api")
///     .and("v1")
///     .and("posts")
///     .and(syntax::param::<i32>())
/// )
/// ```
#[macro_export(local_inner_macros)]
macro_rules! path {
    // with method
    (@$method:ident $($t:tt)*) => (
        $crate::endpoint::IntoEndpointExt::and(
            $crate::endpoint::syntax::verb::$method(),
            path_impl!(@start $($t)*)
        )
    );

    // without method
    (/ $($t:tt)*) => ( path_impl!(@start / $($t)*) );
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! path_impl {
    (@start / $head:tt $(/ $tail:tt)*) => {{
        let __p = path_impl!(@segment $head);
        $(
            let __p = $crate::endpoint::IntoEndpointExt::and(__p, path_impl!(@segment $tail));
        )*
        __p
    }};
    (@start / $head:tt $(/ $tail:tt)* /) => {
        $crate::endpoint::IntoEndpointExt::and(
            path_impl!(@start / $head $(/ $tail)*),
            $crate::endpoint::syntax::eos(),
        )
    };
    (@start /) => ( $crate::endpoint::syntax::eos() );

    (@segment $t:ty) => ( $crate::endpoint::syntax::param::<$t>() );
    (@segment $s:expr) => ( ($s) );
}
