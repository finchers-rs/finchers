use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;
use futures::Future;
use futures::future::{self, ok, FutureResult};

use either::Either;
use endpoint::{Context, Endpoint, EndpointResult, EndpointErrorKind};

pub mod join {
    use endpoint::{Context, Endpoint, EndpointResult};
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


pub struct Or<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Error = E1::Error>,
{
    type Item = Either<E1::Item, E2::Item>;
    type Error = E1::Error;
    type Future = Either<E1::Future, E2::Future>;
    fn apply<'r>(self, ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        let Or(e1, e2) = self;
        e1.apply(ctx.clone())
            .map(|(ctx, a)| (ctx, Either::A(a)))
            .or_else(|_| e2.apply(ctx).map(|(ctx, b)| (ctx, Either::B(b))))
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


pub struct PathSeq<T>(PhantomData<fn(T) -> T>);

impl<T: FromStr> Endpoint for PathSeq<T> {
    type Item = Vec<T>;
    type Error = ();
    type Future = FutureResult<Vec<T>, ()>;

    fn apply<'r>(self, mut ctx: Context<'r>) -> EndpointResult<(Context<'r>, Self::Future)> {
        let seq = ctx.routes.iter().filter_map(|s| s.parse().ok()).collect();
        ctx.routes = Default::default();
        Ok((ctx, ok(seq)))
    }
}

pub fn path_seq<T: FromStr>() -> PathSeq<T> {
    PathSeq(PhantomData)
}
