#![allow(missing_docs)]

use super::{Context, Endpoint, IntoEndpoint};
use futures::future;
use request::Input;
use std::fmt;

pub fn join_all<I>(iter: I) -> JoinAll<<I::Item as IntoEndpoint>::Endpoint>
where
    I: IntoIterator,
    I::Item: IntoEndpoint,
{
    JoinAll {
        inner: iter.into_iter().map(IntoEndpoint::into_endpoint).collect(),
    }
}

pub struct JoinAll<E: Endpoint> {
    inner: Vec<E>,
}

impl<E: Endpoint + Clone> Clone for JoinAll<E> {
    fn clone(&self) -> Self {
        JoinAll {
            inner: self.inner.clone(),
        }
    }
}

impl<E: Endpoint + fmt::Debug> fmt::Debug for JoinAll<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("JoinAll").field(&self.inner).finish()
    }
}

impl<E: Endpoint> Endpoint for JoinAll<E> {
    type Item = Vec<E::Item>;
    type Future = future::JoinAll<Vec<E::Future>>;

    fn apply(&self, input: &Input, ctx: &mut Context) -> Option<Self::Future> {
        let inner: Vec<_> = self.inner.iter().map(|e| e.apply(input, ctx)).collect::<Option<_>>()?;
        Some(future::join_all(inner))
    }
}
