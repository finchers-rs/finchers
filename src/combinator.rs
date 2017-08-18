use std::borrow::Cow;
use futures::{Future, Poll};
use futures::future::{self, ok, FutureResult};
use hyper::Method;

use context::Context;
use either::Either;
use endpoint::Endpoint;
use errors::{EndpointResult, EndpointErrorKind};


pub mod join {
    use context::Context;
    use endpoint::Endpoint;
    use errors::EndpointResult;
    use futures::Future;
    use futures::future::{Join, Join3, Join4, Join5};

    // TODO: use macro to derive implementations.
    impl<A, B> Endpoint for (A, B)
    where
        A: Endpoint,
        B: Endpoint<Error = A::Error>,
    {
        type Item = (A::Item, B::Item);
        type Error = A::Error;
        type Future = Join<A::Future, B::Future>;
        fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
            let (ctx, a) = self.0.apply(ctx)?;
            let (ctx, b) = self.1.apply(ctx)?;
            Ok((ctx, a.join(b)))
        }
    }

    impl<A, B, C> Endpoint for (A, B, C)
    where
        A: Endpoint,
        B: Endpoint<Error = A::Error>,
        C: Endpoint<Error = A::Error>,
    {
        type Item = (A::Item, B::Item, C::Item);
        type Error = A::Error;
        type Future = Join3<A::Future, B::Future, C::Future>;
        fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
            let (ctx, a) = self.0.apply(ctx)?;
            let (ctx, b) = self.1.apply(ctx)?;
            let (ctx, c) = self.2.apply(ctx)?;
            Ok((ctx, a.join3(b, c)))
        }
    }

    impl<A, B, C, D> Endpoint for (A, B, C, D)
    where
        A: Endpoint,
        B: Endpoint<Error = A::Error>,
        C: Endpoint<Error = A::Error>,
        D: Endpoint<Error = A::Error>,
    {
        type Item = (A::Item, B::Item, C::Item, D::Item);
        type Error = A::Error;
        type Future = Join4<A::Future, B::Future, C::Future, D::Future>;
        fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
            let (ctx, a) = self.0.apply(ctx)?;
            let (ctx, b) = self.1.apply(ctx)?;
            let (ctx, c) = self.2.apply(ctx)?;
            let (ctx, d) = self.3.apply(ctx)?;
            Ok((ctx, a.join4(b, c, d)))
        }
    }

    impl<A, B, C, D, E> Endpoint for (A, B, C, D, E)
    where
        A: Endpoint,
        B: Endpoint<Error = A::Error>,
        C: Endpoint<Error = A::Error>,
        D: Endpoint<Error = A::Error>,
        E: Endpoint<Error = A::Error>,
    {
        type Item = (A::Item, B::Item, C::Item, D::Item, E::Item);
        type Error = A::Error;
        type Future = Join5<A::Future, B::Future, C::Future, D::Future, E::Future>;
        fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
            let (ctx, a) = self.0.apply(ctx)?;
            let (ctx, b) = self.1.apply(ctx)?;
            let (ctx, c) = self.2.apply(ctx)?;
            let (ctx, d) = self.3.apply(ctx)?;
            let (ctx, e) = self.4.apply(ctx)?;
            Ok((ctx, a.join5(b, c, d, e)))
        }
    }
}


pub struct With<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for With<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = E2::Item;
    type Error = E2::Error;
    type Future = E2::Future;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        let With(e1, e2) = self;
        e1.apply(ctx).and_then(|(ctx, _)| e2.apply(ctx))
    }
}


pub struct Skip<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for Skip<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = E1::Item;
    type Error = E1::Error;
    type Future = E1::Future;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        let Skip(e1, e2) = self;
        e1.apply(ctx).and_then(|(ctx, f)| {
            e2.apply(ctx).map(|(ctx, _)| (ctx, f))
        })
    }
}


pub struct Map<E, F>(pub(crate) E, pub(crate) F);

impl<E, F, R> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R,
{
    type Item = R;
    type Error = E::Error;
    type Future = future::Map<E::Future, F>;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        let Map(e, f) = self;
        e.apply(ctx).map(|(ctx, fut)| (ctx, fut.map(f)))
    }
}

pub struct MapErr<E, F>(pub(crate) E, pub(crate) F);

impl<E, F, R> Endpoint for MapErr<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Error) -> R,
{
    type Item = E::Item;
    type Error = R;
    type Future = future::MapErr<E::Future, F>;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        let MapErr(e, f) = self;
        e.apply(ctx).map(|(ctx, fut)| (ctx, fut.map_err(f)))
    }
}


pub struct Or<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Item = E1::Item, Error = E1::Error>,
{
    type Item = E1::Item;
    type Error = E1::Error;
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        let Or(e1, e2) = self;
        e1.apply(ctx.clone())
            .map(|(ctx, a)| (ctx, Either::A(a)))
            .or_else(|_| e2.apply(ctx).map(|(ctx, b)| (ctx, Either::B(b))))
            .map(|(ctx, f)| (ctx, OrFuture(f)))
    }
}

pub struct OrFuture<E1, E2>(Either<E1, E2>);

impl<E1, E2> Future for OrFuture<E1, E2>
where
    E1: Future,
    E2: Future<Item = E1::Item, Error = E1::Error>,
{
    type Item = E1::Item;
    type Error = E1::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match try_ready!(self.0.poll()) {
            Either::A(a) => Ok(a.into()),
            Either::B(b) => Ok(b.into()),
        }
    }
}


// --------------------------------------------------------------------------------------

impl<'a> Endpoint for &'a str {
    type Item = ();
    type Error = ();
    type Future = FutureResult<(), ()>;

    fn apply<'r>(self, mut ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        match ctx.routes.get(0) {
            Some(s) if s == self => {}
            _ => return Err(EndpointErrorKind::NoRoute.into()),
        }
        ctx.routes.pop_front();
        Ok((ctx, ok(())))
    }
}

impl Endpoint for String {
    type Item = ();
    type Error = ();
    type Future = FutureResult<(), ()>;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        (&self as &str).apply(ctx)
    }
}

impl<'a> Endpoint for Cow<'a, str> {
    type Item = ();
    type Error = ();
    type Future = FutureResult<(), ()>;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        (&self as &str).apply(ctx)
    }
}


pub mod path {
    use std::marker::PhantomData;
    use std::str::FromStr;
    use futures::future::{ok, FutureResult};

    use context::Context;
    use endpoint::Endpoint;
    use errors::{EndpointResult, EndpointErrorKind};

    pub struct Path<T>(PhantomData<fn(T) -> T>);

    impl<T: FromStr> Endpoint for Path<T> {
        type Item = T;
        type Error = ();
        type Future = FutureResult<T, ()>;

        fn apply<'r>(self, mut ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
            let value: T = match ctx.routes.get(0).and_then(|s| s.parse().ok()) {
                Some(val) => val,
                _ => return Err(EndpointErrorKind::NoRoute.into()),
            };
            ctx.routes.pop_front();
            Ok((ctx, ok(value)))
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
        type Error = ();
        type Future = FutureResult<Vec<T>, ()>;

        fn apply<'r>(self, mut ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
            let seq = ctx.routes
                .iter()
                .map(|s| s.parse())
                .collect::<Result<_, T::Err>>()
                .map_err(|e| e.to_string())?;
            ctx.routes = Default::default();
            Ok((ctx, ok(seq)))
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
        type Error = ();
        type Future = FutureResult<(), ()>;

        fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
            if ctx.routes.len() > 0 {
                return Err(EndpointErrorKind::RemainingPath.into());
            }
            Ok((ctx, ok(())))
        }
    }

    pub fn path_end() -> PathEnd {
        PathEnd
    }

    #[allow(non_upper_case_globals)]
    pub const end_: PathEnd = PathEnd;
}

// ------------------------------------------------------------------

pub struct MatchMethod<E>(Method, E);

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Future = E::Future;

    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        if ctx.request.method != self.0 {
            return Err(EndpointErrorKind::InvalidMethod.into());
        }
        self.1.apply(ctx)
    }
}

pub fn get<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Get, endpoint)
}

pub fn post<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Post, endpoint)
}
