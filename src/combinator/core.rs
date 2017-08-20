//! Definition of core combinators

use futures::{Future, Poll};
use futures::future::{self, Join, Join3, Join4, Join5};

use context::Context;
use endpoint::Endpoint;
use errors::EndpointResult;
use request::Body;


impl<A, B> Endpoint for (A, B)
where
    A: Endpoint,
    B: Endpoint,
{
    type Item = (A::Item, B::Item);
    type Future = Join<A::Future, B::Future>;

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let (ctx, body, a) = self.0.apply(ctx, body)?;
        let (ctx, body, b) = self.1.apply(ctx, body)?;
        Ok((ctx, body, a.join(b)))
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

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let (ctx, body, a) = self.0.apply(ctx, body)?;
        let (ctx, body, b) = self.1.apply(ctx, body)?;
        let (ctx, body, c) = self.2.apply(ctx, body)?;
        Ok((ctx, body, a.join3(b, c)))
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

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let (ctx, body, a) = self.0.apply(ctx, body)?;
        let (ctx, body, b) = self.1.apply(ctx, body)?;
        let (ctx, body, c) = self.2.apply(ctx, body)?;
        let (ctx, body, d) = self.3.apply(ctx, body)?;
        Ok((ctx, body, a.join4(b, c, d)))
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

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let (ctx, body, a) = self.0.apply(ctx, body)?;
        let (ctx, body, b) = self.1.apply(ctx, body)?;
        let (ctx, body, c) = self.2.apply(ctx, body)?;
        let (ctx, body, d) = self.3.apply(ctx, body)?;
        let (ctx, body, e) = self.4.apply(ctx, body)?;
        Ok((ctx, body, a.join5(b, c, d, e)))
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

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let With(e1, e2) = self;
        e1.apply(ctx, body).and_then(
            |(ctx, body, _)| e2.apply(ctx, body),
        )
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

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let Skip(e1, e2) = self;
        e1.apply(ctx, body).and_then(|(ctx, body, f)| {
            e2.apply(ctx, body).map(|(ctx, body, _)| (ctx, body, f))
        })
    }
}


#[allow(missing_docs)]
pub struct Map<E, F>(pub(crate) E, pub(crate) F);

impl<E, F, R> Endpoint for Map<E, F>
where
    E: Endpoint,
    F: FnOnce(E::Item) -> R,
{
    type Item = R;
    type Future = future::Map<E::Future, F>;

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let Map(e, f) = self;
        e.apply(ctx, body).map(
            |(ctx, body, fut)| (ctx, body, fut.map(f)),
        )
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

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        let Or(e1, e2) = self;
        e1.apply(ctx.clone(), body)
            .map(|(ctx, body, a)| (ctx, body, OrFuture::A(a)))
            .or_else(|(_, body)| {
                e2.apply(ctx, body).map(|(ctx, body, b)| {
                    (ctx, body, OrFuture::B(b))
                })
            })
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
