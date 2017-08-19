use context::Context;
use endpoint::Endpoint;
use errors::EndpointResult;
use futures::Future;
use futures::future::{Join, Join3, Join4, Join5};

// TODO: use macro to derive implementations.
impl<A, B> Endpoint for (A, B)
where
    A: Endpoint,
    B: Endpoint,
{
    type Item = (A::Item, B::Item);
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
    B: Endpoint,
    C: Endpoint,
{
    type Item = (A::Item, B::Item, C::Item);
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
    B: Endpoint,
    C: Endpoint,
    D: Endpoint,
{
    type Item = (A::Item, B::Item, C::Item, D::Item);
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
    B: Endpoint,
    C: Endpoint,
    D: Endpoint,
    E: Endpoint,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item);
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
