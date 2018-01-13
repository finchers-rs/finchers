#![allow(missing_docs)]

use std::fmt;
use super::{Endpoint, EndpointContext, IntoEndpoint};
use super::task;

pub fn join_all<I, E, A, B>(iter: I) -> JoinAll<E::Endpoint>
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
    type Task = task::join_all::JoinAll<E::Task>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        let inner: Vec<E::Task> = try_opt!(
            self.inner
                .iter()
                .map(|e| e.apply(ctx))
                .collect::<Option<_>>()
        );
        Some(task::join_all::JoinAll { inner })
    }
}
