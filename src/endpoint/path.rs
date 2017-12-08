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
pub struct Path<T>(PhantomData<fn(T) -> T>);

impl<T> Copy for Path<T> {}

impl<T> Clone for Path<T> {
    fn clone(&self) -> Path<T> {
        *self
    }
}

impl<T: FromParam> Endpoint for Path<T> {
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
pub fn path<T: FromParam>() -> Path<T> {
    Path(PhantomData)
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct PathSeq<I, T>(PhantomData<fn() -> (I, T)>);

impl<I, T> Clone for PathSeq<I, T> {
    fn clone(&self) -> PathSeq<I, T> {
        PathSeq(PhantomData)
    }
}

impl<I, T> Copy for PathSeq<I, T> {}

impl<I, T> Endpoint for PathSeq<I, T>
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
pub fn path_seq<I, T>() -> PathSeq<I, T>
where
    I: FromIterator<T>,
    T: FromParam,
{
    PathSeq(PhantomData)
}

#[allow(missing_docs)]
pub type PathVec<T> = PathSeq<Vec<T>, T>;

/// Equivalent to `path_seq<Vec<T>, T>()`
pub fn path_vec<T: FromParam>() -> PathVec<T> {
    PathSeq(PhantomData)
}


#[allow(missing_docs)]
pub trait PathExt: FromParam {
    /// equivalent to `path::<Self>()`
    const PATH: Path<Self> = Path(PhantomData);
}

impl<T: FromParam> PathExt for T {}
