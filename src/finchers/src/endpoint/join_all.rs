#![allow(missing_docs)]

use std::fmt;
use futures::future;
use super::{Endpoint, EndpointContext, EndpointResult, Input, IntoEndpoint};

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
    type Result = JoinAllResult<E::Result>;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let inner = try_opt!(
            self.inner
                .iter()
                .map(|e| e.apply(input, ctx))
                .collect::<Option<_>>()
        );
        Some(JoinAllResult { inner })
    }
}

#[derive(Debug)]
pub struct JoinAllResult<T> {
    inner: Vec<T>,
}

impl<T: EndpointResult> EndpointResult for JoinAllResult<T> {
    type Item = Vec<T::Item>;
    type Future = future::JoinAll<Vec<T::Future>>;

    fn into_future(self, input: &mut Input) -> Self::Future {
        future::join_all(
            self.inner
                .into_iter()
                .map(|t| t.into_future(input))
                .collect(),
        )
    }
}
