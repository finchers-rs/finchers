//! Definition of endpoints to parse path segments

use std::borrow::Cow;
use std::iter::FromIterator;
use std::marker::PhantomData;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};
use request::FromParam;
use util::NoReturn;


#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct PathSegment<'a>(Cow<'a, str>);

impl<'a> Endpoint for PathSegment<'a> {
    type Item = ();
    type Error = NoReturn;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        if !ctx.next_segment().map(|s| s == self.0).unwrap_or(false) {
            return Err(EndpointError::Skipped);
        }
        Ok(ok(()))
    }
}

/// Create an endpoint which represents a path segment
#[inline(always)]
pub fn segment<'a, T: 'a + Into<Cow<'a, str>>>(segment: T) -> PathSegment<'a> {
    PathSegment(segment.into())
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct PathParam<T>(PhantomData<fn() -> T>);

impl<T> Copy for PathParam<T> {}

impl<T> Clone for PathParam<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: FromParam> Endpoint for PathParam<T> {
    type Item = T;
    type Error = NoReturn;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        match ctx.next_segment().map(|s| T::from_param(s)) {
            Some(Ok(value)) => Ok(ok(value)),
            _ => return Err(EndpointError::TypeMismatch),
        }
    }
}

/// Create an endpoint which represents a path element
pub fn param<T: FromParam>() -> PathParam<T> {
    PathParam(PhantomData)
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct PathParams<I, T>(PhantomData<fn() -> (I, T)>);

impl<I, T> Copy for PathParams<I, T> {}

impl<I, T> Clone for PathParams<I, T> {
    fn clone(&self) -> Self {
        *self
    }
}


impl<I, T> Endpoint for PathParams<I, T>
where
    I: FromIterator<T> + Default,
    T: FromParam,
{
    type Item = I;
    type Error = NoReturn;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        match ctx.collect_remaining_segments() {
            Some(Ok(seq)) => Ok(ok(seq)),
            Some(Err(_)) => Err(EndpointError::TypeMismatch),
            None => Ok(ok(Default::default())),
        }
    }
}

/// Create an endpoint which represents the sequence of remaining path elements
pub fn params<I, T>() -> PathParams<I, T>
where
    I: FromIterator<T>,
    T: FromParam,
{
    PathParams(PhantomData)
}


#[allow(missing_docs)]
#[deprecated(since = "0.6.0", note = "use param::<T>() instead")]
pub trait PathExt: FromParam {
    /// equivalent to `path::<Self>()`
    const PATH: PathParam<Self> = PathParam(PhantomData);
}

#[allow(deprecated)]
impl<T: FromParam> PathExt for T {}
