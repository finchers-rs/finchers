use std::fmt;

use crate::common::Tuple;
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{EndpointFuture, Poll, TaskContext};

#[allow(missing_debug_implementations)]
pub struct EndpointFutureObj<T>(Box<dyn EndpointFuture<Output = T> + Send + 'static>);

impl<T> EndpointFuture for EndpointFutureObj<T> {
    type Output = T;

    #[inline]
    fn poll_endpoint(&mut self, cx: &mut TaskContext<'_>) -> Poll<Self::Output, Error> {
        self.0.poll_endpoint(cx)
    }
}

trait FutureObjEndpoint {
    type Output: Tuple;

    fn apply_obj(&self, ecx: &mut ApplyContext<'_>)
        -> ApplyResult<EndpointFutureObj<Self::Output>>;
}

impl<E> FutureObjEndpoint for E
where
    E: Endpoint,
    E::Future: Send + 'static,
{
    type Output = E::Output;

    #[inline(always)]
    fn apply_obj(
        &self,
        ecx: &mut ApplyContext<'_>,
    ) -> ApplyResult<EndpointFutureObj<Self::Output>> {
        let future = self.apply(ecx)?;
        Ok(EndpointFutureObj(Box::new(future)))
    }
}

#[allow(missing_docs)]
pub struct EndpointObj<T: Tuple + 'static> {
    inner: Box<dyn FutureObjEndpoint<Output = T> + Send + Sync + 'static>,
}

impl<T: Tuple + 'static> EndpointObj<T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> EndpointObj<T>
    where
        E: Endpoint<Output = T> + Send + Sync + 'static,
        E::Future: Send + 'static,
    {
        EndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<T: Tuple + 'static> fmt::Debug for EndpointObj<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("EndpointObj").finish()
    }
}

impl<T: Tuple + 'static> Endpoint for EndpointObj<T> {
    type Output = T;
    type Future = EndpointFutureObj<T>;

    #[inline(always)]
    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        self.inner.apply_obj(ecx)
    }
}

// ==== BoxedLocal ====
#[allow(missing_debug_implementations)]
pub struct LocalEndpointFutureObj<T>(Box<dyn EndpointFuture<Output = T> + 'static>);

impl<T> EndpointFuture for LocalEndpointFutureObj<T> {
    type Output = T;

    #[inline]
    fn poll_endpoint(&mut self, cx: &mut TaskContext<'_>) -> Poll<Self::Output, Error> {
        self.0.poll_endpoint(cx)
    }
}

trait LocalFutureObjEndpoint {
    type Output: Tuple;

    fn apply_local_obj(
        &self,
        ecx: &mut ApplyContext<'_>,
    ) -> ApplyResult<LocalEndpointFutureObj<Self::Output>>;
}

impl<E: Endpoint> LocalFutureObjEndpoint for E
where
    E::Future: 'static,
{
    type Output = E::Output;

    #[inline(always)]
    fn apply_local_obj(
        &self,
        ecx: &mut ApplyContext<'_>,
    ) -> ApplyResult<LocalEndpointFutureObj<Self::Output>> {
        let future = self.apply(ecx)?;
        Ok(LocalEndpointFutureObj(Box::new(future)))
    }
}

#[allow(missing_docs)]
pub struct LocalEndpointObj<T: Tuple + 'static> {
    inner: Box<dyn LocalFutureObjEndpoint<Output = T> + 'static>,
}

impl<T: Tuple + 'static> LocalEndpointObj<T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> LocalEndpointObj<T>
    where
        E: Endpoint<Output = T> + Send + Sync + 'static,
    {
        LocalEndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<T: Tuple + 'static> fmt::Debug for LocalEndpointObj<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("LocalEndpointObj").finish()
    }
}

impl<T: Tuple + 'static> Endpoint for LocalEndpointObj<T> {
    type Output = T;
    type Future = LocalEndpointFutureObj<T>;

    #[inline(always)]
    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        self.inner.apply_local_obj(ecx)
    }
}
