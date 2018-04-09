use finchers_core::Input;
use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use futures::{future, IntoFuture};

pub fn new<E1, E2>(e1: E1, e2: E2) -> And<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
{
    And {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct And<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for And<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = (E1::Item, E2::Item);
    type Future = future::Join<E1::Future, E2::Future>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let f1 = self.e1.apply(input, ctx)?;
        let f2 = self.e2.apply(input, ctx)?;
        Some(IntoFuture::into_future((f1, f2)))
    }
}
