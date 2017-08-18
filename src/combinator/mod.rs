pub mod join;
pub mod path;
pub mod method;

use std::borrow::Cow;
use futures::{Future, Poll};
use futures::future::{self, ok, FutureResult};

use context::Context;
use either::Either;
use endpoint::Endpoint;
use errors::{EndpointResult, EndpointErrorKind};


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
