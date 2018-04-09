use finchers_core::Input;
use {Context, Endpoint, IntoEndpoint};

pub fn new<E1, E2>(e1: E1, e2: E2) -> Right<E1::Endpoint, E2::Endpoint>
where
    E1: IntoEndpoint,
    E2: IntoEndpoint,
{
    Right {
        e1: e1.into_endpoint(),
        e2: e2.into_endpoint(),
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Right<E1, E2> {
    e1: E1,
    e2: E2,
}

impl<E1, E2> Endpoint for Right<E1, E2>
where
    E1: Endpoint,
    E2: Endpoint,
{
    type Item = E2::Item;
    type Future = E2::Future;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let _f1 = self.e1.apply(input, ctx)?;
        let f2 = self.e2.apply(input, ctx)?;
        Some(f2)
    }
}
