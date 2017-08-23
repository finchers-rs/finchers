//! Definition of combinators

pub mod either;

use std::sync::Arc;
use futures::{Future, Poll};
use futures::future::{Join, Join3, Join4, Join5};

use context::Context;
use endpoint::{Endpoint, EndpointResult};
use self::either::Either2;


impl<A, B> Endpoint for (A, B)
where
    A: Endpoint,
    B: Endpoint,
{
    type Item = (A::Item, B::Item);
    type Future = Join<A::Future, B::Future>;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let a = self.0.apply(ctx)?;
        let b = self.1.apply(ctx)?;
        Ok(a.join(b))
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

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let a = self.0.apply(ctx)?;
        let b = self.1.apply(ctx)?;
        let c = self.2.apply(ctx)?;
        Ok(a.join3(b, c))
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

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let a = self.0.apply(ctx)?;
        let b = self.1.apply(ctx)?;
        let c = self.2.apply(ctx)?;
        let d = self.3.apply(ctx)?;
        Ok(a.join4(b, c, d))
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

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let a = self.0.apply(ctx)?;
        let b = self.1.apply(ctx)?;
        let c = self.2.apply(ctx)?;
        let d = self.3.apply(ctx)?;
        let e = self.4.apply(ctx)?;
        Ok(a.join5(b, c, d, e))
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

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let _ = self.0.apply(ctx)?;
        let b = self.1.apply(ctx)?;
        Ok(b)
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

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let a = self.0.apply(ctx)?;
        let _ = self.1.apply(ctx)?;
        Ok(a)
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

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let inner = self.0.apply(ctx)?;
        let map_fn = self.1.clone();
        Ok(MapFuture { inner, map_fn })
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
    E2: Endpoint,
{
    type Item = Either2<E1::Item, E2::Item>;
    type Future = Either2<E1::Future, E2::Future>;

    fn apply(&self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let mut ctx1 = ctx.clone();
        if let Ok(f) = self.0.apply(&mut ctx1) {
            *ctx = ctx1;
            return Ok(Either2::E1(f));
        }

        self.1.apply(ctx).map(Either2::E2)
    }
}
