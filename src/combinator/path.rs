//! Definition of endpoints to parse path segments

use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};

use context::Context;
use endpoint::Endpoint;
use errors::*;


impl<'a> Endpoint for &'a str {
    type Item = ();
    type Future = FutureResult<(), FinchersError>;

    fn apply<'r, 'b>(self, mut ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let matched = match ctx.routes.get(0) {
            Some(s) if s == &self => true,
            _ => false,
        };
        if !matched {
            return (ctx, Err(FinchersErrorKind::Routing.into()));
        }

        ctx.routes.pop_front();
        (ctx, Ok(ok(())))
    }
}

impl Endpoint for String {
    type Item = ();
    type Future = FutureResult<(), FinchersError>;

    fn apply<'r, 'b>(self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        (&self as &str).apply(ctx)
    }
}

impl<'a> Endpoint for Cow<'a, str> {
    type Item = ();
    type Future = FutureResult<(), FinchersError>;

    fn apply<'r, 'b>(self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        (&self as &str).apply(ctx)
    }
}


#[allow(missing_docs)]
pub struct Path<T>(PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for Path<T> {
    type Item = T;
    type Future = FutureResult<T, FinchersError>;

    fn apply<'r, 'b>(self, mut ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let value: T = match ctx.routes.get(0).and_then(|s| s.parse().ok()) {
            Some(val) => val,
            _ => return (ctx, Err(FinchersErrorKind::Routing.into())),
        };
        ctx.routes.pop_front();
        (ctx, Ok(ok(value)))
    }
}

/// Create a combinator, to take a segment of path and then converting it to `T`
pub fn path<T: FromStr>() -> Path<T> {
    Path(PhantomData)
}

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
pub struct PathSeq<T>(PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for PathSeq<T>
where
    T::Err: ::std::fmt::Display,
{
    type Item = Vec<T>;
    type Future = FutureResult<Vec<T>, FinchersError>;

    fn apply<'r, 'b>(self, mut ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let seq = match ctx.routes
            .iter()
            .map(|s| s.parse())
            .collect::<Result<_, T::Err>>() {
            Ok(seq) => seq,
            Err(e) => return (ctx, Err(e.to_string().into())),
        };
        ctx.routes = Default::default();
        (ctx, Ok(ok(seq)))
    }
}

/// Create a combinator, to take the remaining segments of path and then converting them to `T`
pub fn path_vec<T: FromStr>() -> PathSeq<T> {
    PathSeq(PhantomData)
}

/// Equivalent to `path_vec::<i32>()`
#[allow(non_upper_case_globals)]
pub const i32_vec_: PathSeq<i32> = PathSeq(PhantomData);

/// Equivalent to `path_vec::<u32>()`
#[allow(non_upper_case_globals)]
pub const u32_vec_: PathSeq<u32> = PathSeq(PhantomData);

/// Equivalent to `path_vec::<i64>()`
#[allow(non_upper_case_globals)]
pub const i64_vec_: PathSeq<i64> = PathSeq(PhantomData);

/// Equivalent to `path_vec::<u64>()`
#[allow(non_upper_case_globals)]
pub const u64_vec_: PathSeq<u64> = PathSeq(PhantomData);

/// Equivalent to `path_vec::<f32>()`
#[allow(non_upper_case_globals)]
pub const f32_vec_: PathSeq<f32> = PathSeq(PhantomData);

/// Equivalent to `path_vec::<f64>()`
#[allow(non_upper_case_globals)]
pub const f64_vec_: PathSeq<f64> = PathSeq(PhantomData);

/// Equivalent to `path_vec::<String>()`
#[allow(non_upper_case_globals)]
pub const string_vec_: PathSeq<String> = PathSeq(PhantomData);


#[allow(missing_docs)]
pub struct PathEnd;

impl Endpoint for PathEnd {
    type Item = ();
    type Future = FutureResult<(), FinchersError>;

    fn apply<'r, 'b>(self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        if ctx.routes.len() > 0 {
            return (ctx, Err(FinchersErrorKind::Routing.into()));
        }
        (ctx, Ok(ok(())))
    }
}

/// Create a combinator to check the end of segments
pub fn path_end() -> PathEnd {
    PathEnd
}

/// Equivalent to `path_end()`
#[allow(non_upper_case_globals)]
pub const end_: PathEnd = PathEnd;
