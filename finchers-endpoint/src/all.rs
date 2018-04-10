use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use futures::future;

pub fn all<I>(iter: I) -> All<<I::Item as IntoEndpoint>::Endpoint>
where
    I: IntoIterator,
    I::Item: IntoEndpoint,
    <I::Item as IntoEndpoint>::Item: Send,
{
    All {
        inner: iter.into_iter().map(IntoEndpoint::into_endpoint).collect(),
    }
}

#[derive(Clone, Debug)]
pub struct All<E> {
    inner: Vec<E>,
}

impl<E> Endpoint for All<E>
where
    E: Endpoint,
    E::Item: Send,
{
    type Item = Vec<E::Item>;
    type Future = future::JoinAll<Vec<E::Future>>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        let inner: Vec<_> = self.inner.iter().map(|e| e.apply(cx)).collect::<Option<_>>()?;
        Some(future::join_all(inner))
    }
}
