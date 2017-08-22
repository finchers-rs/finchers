//! Definition of endpoints to parse path segments

use std::borrow::Cow;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::Endpoint;
use errors::*;


impl<'a> Endpoint for &'a str {
    type Item = ();
    type Future = FutureResult<(), FinchersError>;

    fn apply<'r, 'b>(&self, mut ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let matched = match ctx.routes.get(0) {
            Some(s) if s == self => true,
            _ => false,
        };
        if !matched {
            return (ctx, Err(FinchersErrorKind::NotFound.into()));
        }

        ctx.routes.pop_front();
        (ctx, Ok(ok(())))
    }
}

impl Endpoint for String {
    type Item = ();
    type Future = FutureResult<(), FinchersError>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        (self as &str).apply(ctx)
    }
}

impl<'a> Endpoint for Cow<'a, str> {
    type Item = ();
    type Future = FutureResult<(), FinchersError>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        (&*self as &str).apply(ctx)
    }
}


#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Path<T>(PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for Path<T> {
    type Item = T;
    type Future = FutureResult<T, FinchersError>;

    fn apply<'r, 'b>(&self, mut ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let value: T = match ctx.routes.get(0).and_then(|s| s.parse().ok()) {
            Some(val) => val,
            _ => return (ctx, Err(FinchersErrorKind::NotFound.into())),
        };
        ctx.routes.pop_front();
        (ctx, Ok(ok(value)))
    }
}

/// Create an endpoint which represents a path element
pub fn path<T: FromStr>() -> Path<T> {
    Path(PhantomData)
}

/// Equivalent to `path::<i8>()`
#[allow(non_upper_case_globals)]
pub const i8_: Path<i8> = Path(PhantomData);

/// Equivalent to `path::<u8>()`
#[allow(non_upper_case_globals)]
pub const u8_: Path<u8> = Path(PhantomData);

/// Equivalent to `path::<i16>()`
#[allow(non_upper_case_globals)]
pub const i16_: Path<i16> = Path(PhantomData);

/// Equivalent to `path::<u16>()`
#[allow(non_upper_case_globals)]
pub const u16_: Path<u16> = Path(PhantomData);

/// Equivalent to `path::<i32>()`
#[allow(non_upper_case_globals)]
pub const i32_: Path<i32> = Path(PhantomData);

/// Equivalent to `path::<u32>()`
#[allow(non_upper_case_globals)]
pub const u32_: Path<u32> = Path(PhantomData);

/// Equivalent to `path::<i64>()`
#[allow(non_upper_case_globals)]
pub const i64_: Path<i64> = Path(PhantomData);

/// Equivalent to `path::<u64>()`
#[allow(non_upper_case_globals)]
pub const u64_: Path<u64> = Path(PhantomData);

/// Equivalent to `path::<isize>()`
#[allow(non_upper_case_globals)]
pub const isize_: Path<isize> = Path(PhantomData);

/// Equivalent to `path::<usize>()`
#[allow(non_upper_case_globals)]
pub const usize_: Path<usize> = Path(PhantomData);

/// Equivalent to `path::<f32>()`
#[allow(non_upper_case_globals)]
pub const f32_: Path<f32> = Path(PhantomData);

/// Equivalent to `path::<f64>()`
#[allow(non_upper_case_globals)]
pub const f64_: Path<f64> = Path(PhantomData);

/// Equivalent to `path::<String>()`
#[allow(non_upper_case_globals)]
pub const string_: Path<String> = Path(PhantomData);


#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct PathSeq<I, T>(PhantomData<fn() -> (I, T)>);

impl<I, T> Endpoint for PathSeq<I, T>
where
    I: FromIterator<T>,
    T: FromStr,
{
    type Item = I;
    type Future = FutureResult<I, FinchersError>;

    fn apply<'r, 'b>(&self, mut ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let seq = match ctx.routes
            .iter()
            .map(|s| s.parse())
            .collect::<Result<_, T::Err>>()
        {
            Ok(seq) => seq,
            Err(_) => return (ctx, Err(FinchersErrorKind::NotFound.into())),
        };
        ctx.routes = Default::default();
        (ctx, Ok(ok(seq)))
    }
}

/// Create an endpoint which represents the sequence of remaining path elements
pub fn path_seq<I, T>() -> PathSeq<I, T>
where
    I: FromIterator<T>,
    T: FromStr,
{
    PathSeq(PhantomData)
}

#[allow(missing_docs)]
pub type PathVec<T> = PathSeq<Vec<T>, T>;

/// Equivalent to `path_seq<Vec<T>, T>()`
pub fn path_vec<T: FromStr>() -> PathVec<T> {
    PathSeq(PhantomData)
}
