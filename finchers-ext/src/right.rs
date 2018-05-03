#![allow(missing_docs)]

use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};

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
    type Output = E2::Output;
    type Task = E2::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        drop(self.e1.apply(cx)?);
        let f2 = self.e2.apply(cx)?;
        Some(f2)
    }
}
