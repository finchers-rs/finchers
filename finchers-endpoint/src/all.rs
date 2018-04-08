#![allow(missing_docs)]

use finchers_core::Input;
use futures::future;
use {Context, Endpoint, IntoEndpoint};

pub fn all<I>(iter: I) -> All<<I::Item as IntoEndpoint>::Endpoint>
where
    I: IntoIterator,
    I::Item: IntoEndpoint,
{
    All {
        inner: iter.into_iter().map(IntoEndpoint::into_endpoint).collect(),
    }
}

#[derive(Clone, Debug)]
pub struct All<E> {
    inner: Vec<E>,
}

impl<E: Endpoint> Endpoint for All<E> {
    type Item = Vec<E::Item>;
    type Future = future::JoinAll<Vec<E::Future>>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let inner: Vec<_> = self.inner.iter().map(|e| e.apply(input, ctx)).collect::<Option<_>>()?;
        Some(future::join_all(inner))
    }
}
