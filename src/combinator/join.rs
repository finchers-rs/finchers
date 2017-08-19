use futures::Future;
use futures::future::{Join, Join3, Join4, Join5};

use context::Context;
use endpoint::Endpoint;
use errors::EndpointResult;
use request::Body;


// TODO: use macro to derive implementations.
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
