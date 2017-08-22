//! Definition of combinators

use std::sync::Arc;
use futures::{Future, Poll};
use futures::future::{Join, Join3, Join4, Join5};

use context::Context;
use endpoint::Endpoint;
use errors::FinchersResult;

macro_rules! try_second {
    ($e:expr) => {
        {
            match $e {
                (ctx, Ok(a)) => (ctx, a),
                (ctx, Err(b)) => return (ctx, Err(b.into())),
            }
        }
    }
}

impl<A, B> Endpoint for (A, B)
where
    A: Endpoint,
    B: Endpoint,
{
    type Item = (A::Item, B::Item);
    type Future = Join<A::Future, B::Future>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let (ctx, a) = try_second!(self.0.apply(ctx));
        let (ctx, b) = try_second!(self.1.apply(ctx));
        (ctx, Ok(a.join(b)))
    }
}

impl<A, B, C> Endpoint for (A, B, C)
where
    A: Endpoint,
    B: Endpoint,
    C: Endpoint,
{
    type Item = (A::Item, B::Item, C::Item);
    type Future = Join3<A::Future, B::Future, C::Future>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let (ctx, a) = try_second!(self.0.apply(ctx));
        let (ctx, b) = try_second!(self.1.apply(ctx));
        let (ctx, c) = try_second!(self.2.apply(ctx));
        (ctx, Ok(a.join3(b, c)))
    }
}

impl<A, B, C, D> Endpoint for (A, B, C, D)
where
    A: Endpoint,
    B: Endpoint,
    C: Endpoint,
    D: Endpoint,
{
    type Item = (A::Item, B::Item, C::Item, D::Item);
    type Future = Join4<A::Future, B::Future, C::Future, D::Future>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let (ctx, a) = try_second!(self.0.apply(ctx));
        let (ctx, b) = try_second!(self.1.apply(ctx));
        let (ctx, c) = try_second!(self.2.apply(ctx));
        let (ctx, d) = try_second!(self.3.apply(ctx));
        (ctx, Ok(a.join4(b, c, d)))
    }
}

impl<A, B, C, D, E> Endpoint for (A, B, C, D, E)
where
    A: Endpoint,
    B: Endpoint,
    C: Endpoint,
    D: Endpoint,
    E: Endpoint,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item);
    type Future = Join5<A::Future, B::Future, C::Future, D::Future, E::Future>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let (ctx, a) = try_second!(self.0.apply(ctx));
        let (ctx, b) = try_second!(self.1.apply(ctx));
        let (ctx, c) = try_second!(self.2.apply(ctx));
        let (ctx, d) = try_second!(self.3.apply(ctx));
        let (ctx, e) = try_second!(self.4.apply(ctx));
        (ctx, Ok(a.join5(b, c, d, e)))
    }
}


#[allow(missing_docs)]
pub struct With<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for With<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = E2::Item;
    type Future = E2::Future;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let (ctx, _a) = try_second!(self.0.apply(ctx));
        let (ctx, b) = try_second!(self.1.apply(ctx));
        (ctx, Ok(b))
    }
}


#[allow(missing_docs)]
pub struct Skip<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for Skip<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = E1::Item;
    type Future = E1::Future;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let (ctx, a) = try_second!(self.0.apply(ctx));
        let (ctx, _b) = try_second!(self.1.apply(ctx));
        (ctx, Ok(a))
    }
}


#[allow(missing_docs)]
pub struct Map<E, F>(pub(crate) E, pub(crate) Arc<F>);

impl<E, F, R> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: Fn(E::Item) -> R,
{
    type Item = R;
    type Future = MapFuture<E::Future, F>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let (ctx, inner) = try_second!(self.0.apply(ctx));
        let map_fn = self.1.clone();
        (ctx, Ok(MapFuture { inner, map_fn }))
    }
}

#[doc(hidden)]
pub struct MapFuture<F, M> {
    inner: F,
    map_fn: Arc<M>,
}

impl<F, M, R> Future for MapFuture<F, M>
where
    F: Future,
    M: Fn(F::Item) -> R,
{
    type Item = R;
    type Error = F::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let item = try_ready!(self.inner.poll());
        Ok((*self.map_fn)(item).into())
    }
}


#[allow(missing_docs)]
pub struct Or<E1, E2>(pub(crate) E1, pub(crate) E2);

impl<E1, E2> Endpoint for Or<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint<Item = E1::Item>,
{
    type Item = E1::Item;
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply<'r, 'b>(&self, ctx: Context<'r, 'b>) -> (Context<'r, 'b>, FinchersResult<Self::Future>) {
        let &Or(ref e1, ref e2) = self;
        match e1.apply(ctx.clone()) {
            (ctx, Ok(a)) => (ctx, Ok(OrFuture::A(a))),
            (_ctx, Err(_)) => {
                let (ctx, b) = try_second!(e2.apply(ctx));
                (ctx, Ok(OrFuture::B(b)))
            }
        }
    }
}

#[allow(missing_docs)]
pub enum OrFuture<A, B> {
    A(A),
    B(B),
}

impl<E1, E2> Future for OrFuture<E1, E2>
where
    E1: Future,
    E2: Future<Item = E1::Item, Error = E1::Error>,
{
    type Item = E1::Item;
    type Error = E1::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            OrFuture::A(ref mut a) => a.poll(),
            OrFuture::B(ref mut b) => b.poll(),
        }
    }
}
