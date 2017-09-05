//! Definition of endpoints to parse path segments

use std::borrow::Cow;
use std::iter::FromIterator;
use std::marker::PhantomData;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};
use util::NoReturn;


impl<'a> Endpoint for &'a str {
    type Item = ();
    type Error = NoReturn;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        if !ctx.next_segment().map(|s| s == self).unwrap_or(false) {
            return Err(EndpointError::Skipped);
        }
        Ok(ok(()))
    }
}

impl Endpoint for String {
    type Item = ();
    type Error = NoReturn;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        (&self as &str).apply(ctx)
    }
}

impl<'a> Endpoint for Cow<'a, str> {
    type Item = ();
    type Error = NoReturn;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        (&*self as &str).apply(ctx)
    }
}


#[allow(missing_docs)]
#[derive(Debug)]
pub struct Path<T>(PhantomData<fn(T) -> T>);

impl<T> Clone for Path<T> {
    fn clone(&self) -> Path<T> {
        Path(PhantomData)
    }
}

impl<T> Copy for Path<T> {}

impl<T: FromPath> Endpoint for Path<T> {
    type Item = T;
    type Error = NoReturn;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let value = match ctx.next_segment().and_then(T::from_path) {
            Some(val) => val,
            _ => return Err(EndpointError::TypeMismatch),
        };
        Ok(ok(value))
    }
}

/// Create an endpoint which represents a path element
pub fn path<T: FromPath>() -> Path<T> {
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
    T: FromPath,
{
    type Item = I;
    type Error = NoReturn;
    type Future = FutureResult<Self::Item, Self::Error>;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        ctx.collect_remaining_segments()
            .unwrap_or_else(|| Some(Default::default()))
            .map(ok)
            .ok_or(EndpointError::TypeMismatch)
    }
}

/// Create an endpoint which represents the sequence of remaining path elements
pub fn path_seq<I, T>() -> PathSeq<I, T>
where
    I: FromIterator<T>,
    T: FromPath,
{
    PathSeq(PhantomData)
}

#[allow(missing_docs)]
pub type PathVec<T> = PathSeq<Vec<T>, T>;

/// Equivalent to `path_seq<Vec<T>, T>()`
pub fn path_vec<T: FromPath>() -> PathVec<T> {
    PathSeq(PhantomData)
}


#[allow(missing_docs)]
pub trait PathExt: FromPath {
    /// equivalent to `path::<Self>()`
    const PATH: Path<Self> = Path(PhantomData);
}

impl<T: FromPath> PathExt for T {}


/// Represents the conversion from a path segment
pub trait FromPath: Sized {
    /// Try to convert a `str` to itself
    fn from_path(s: &str) -> Option<Self>;
}

macro_rules! impl_from_path {
    ($($t:ty),*) => {$(
        impl FromPath for $t {
            fn from_path(s: &str) -> Option<Self> {
                s.parse().ok()
            }
        }
    )*}
}

impl_from_path!(
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    isize,
    usize,
    f32,
    f64,
    String
);
