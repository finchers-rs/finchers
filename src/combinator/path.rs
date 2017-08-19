use std::marker::PhantomData;
use std::str::FromStr;
use futures::future::{ok, FutureResult};
use hyper::StatusCode;

use context::Context;
use endpoint::Endpoint;
use errors::{EndpointResult, EndpointErrorKind};
use request::Body;


pub struct Path<T>(PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for Path<T> {
    type Item = T;
    type Future = FutureResult<T, StatusCode>;

    fn apply<'r>(
        self,
        mut ctx: Context<'r>,
        body: Option<Body>,
    ) -> EndpointResult<'r, Self::Future> {
        let value: T = match ctx.routes.get(0).and_then(|s| s.parse().ok()) {
            Some(val) => val,
            _ => return Err((EndpointErrorKind::NoRoute.into(), body)),
        };
        ctx.routes.pop_front();
        Ok((ctx, body, ok(value)))
    }
}

pub fn path<T: FromStr>() -> Path<T> {
    Path(PhantomData)
}

#[allow(non_upper_case_globals)]
pub const i32_: Path<i32> = Path(PhantomData);
#[allow(non_upper_case_globals)]
pub const u32_: Path<u32> = Path(PhantomData);

#[allow(non_upper_case_globals)]
pub const i64_: Path<i64> = Path(PhantomData);
#[allow(non_upper_case_globals)]
pub const u64_: Path<u64> = Path(PhantomData);

#[allow(non_upper_case_globals)]
pub const f32_: Path<f32> = Path(PhantomData);
#[allow(non_upper_case_globals)]
pub const f64_: Path<f64> = Path(PhantomData);

#[allow(non_upper_case_globals)]
pub const string_: Path<String> = Path(PhantomData);



pub struct PathSeq<T>(PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for PathSeq<T>
where
    T::Err: ::std::fmt::Display,
{
    type Item = Vec<T>;
    type Future = FutureResult<Vec<T>, StatusCode>;

    fn apply<'r>(
        self,
        mut ctx: Context<'r>,
        body: Option<Body>,
    ) -> EndpointResult<'r, Self::Future> {
        let seq = match ctx.routes
            .iter()
            .map(|s| s.parse())
            .collect::<Result<_, T::Err>>() {
            Ok(seq) => seq,
            Err(e) => return Err((e.to_string().into(), body)),
        };
        ctx.routes = Default::default();
        Ok((ctx, body, ok(seq)))
    }
}

pub fn path_vec<T: FromStr>() -> PathSeq<T> {
    PathSeq(PhantomData)
}

#[allow(non_upper_case_globals)]
pub const i32_vec_: PathSeq<i32> = PathSeq(PhantomData);
#[allow(non_upper_case_globals)]
pub const u32_vec_: PathSeq<u32> = PathSeq(PhantomData);

#[allow(non_upper_case_globals)]
pub const i64_vec_: PathSeq<i64> = PathSeq(PhantomData);
#[allow(non_upper_case_globals)]
pub const u64_vec_: PathSeq<u64> = PathSeq(PhantomData);

#[allow(non_upper_case_globals)]
pub const f32_vec_: PathSeq<f32> = PathSeq(PhantomData);
#[allow(non_upper_case_globals)]
pub const f64_vec_: PathSeq<f64> = PathSeq(PhantomData);

#[allow(non_upper_case_globals)]
pub const string_vec_: PathSeq<String> = PathSeq(PhantomData);


pub struct PathEnd;

impl Endpoint for PathEnd {
    type Item = ();
    type Future = FutureResult<(), StatusCode>;

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        if ctx.routes.len() > 0 {
            return Err((EndpointErrorKind::RemainingPath.into(), body));
        }
        Ok((ctx, body, ok(())))
    }
}

pub fn path_end() -> PathEnd {
    PathEnd
}

#[allow(non_upper_case_globals)]
pub const end_: PathEnd = PathEnd;
