#![allow(missing_docs)]

use std::fmt;
use futures::future;
use http::Request;
use super::{Endpoint, EndpointContext, EndpointResult, IntoEndpoint};
use errors::HttpError;

pub fn join_all<I, E, A, B: HttpError>(iter: I) -> JoinAll<E::Endpoint>
where
    I: IntoIterator<Item = E>,
    E: IntoEndpoint<A, B>,
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
    type Error = E::Error;
    type Result = JoinAllResult<E::Result>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        let inner = try_opt!(
            self.inner
                .iter()
                .map(|e| e.apply(ctx))
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
    type Error = T::Error;
    type Future = future::JoinAll<Vec<T::Future>>;

    fn into_future(self, request: &mut Request) -> Self::Future {
        future::join_all(
            self.inner
                .into_iter()
                .map(|t| t.into_future(request))
                .collect(),
        )
    }
}
