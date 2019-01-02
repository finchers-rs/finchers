use std::fmt;

use crate::common::Tuple;
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{Context, EndpointFuture, Poll};

#[allow(missing_debug_implementations)]
pub struct EndpointFutureObj<Bd, T>(Box<dyn EndpointFuture<Bd, Output = T> + Send + 'static>);

impl<Bd, T> EndpointFuture<Bd> for EndpointFutureObj<Bd, T> {
    type Output = T;

    #[inline]
    fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
        self.0.poll_endpoint(cx)
    }
}

trait FutureObjEndpoint<Bd> {
    type Output: Tuple;

    fn apply_obj(
        &self,
        ecx: &mut ApplyContext<'_, Bd>,
    ) -> ApplyResult<EndpointFutureObj<Bd, Self::Output>>;
}

impl<Bd, E> FutureObjEndpoint<Bd> for E
where
    E: Endpoint<Bd>,
    E::Future: Send + 'static,
{
    type Output = E::Output;

    #[inline(always)]
    fn apply_obj(
        &self,
        ecx: &mut ApplyContext<'_, Bd>,
    ) -> ApplyResult<EndpointFutureObj<Bd, Self::Output>> {
        let future = self.apply(ecx)?;
        Ok(EndpointFutureObj(Box::new(future)))
    }
}

#[allow(missing_docs)]
pub struct EndpointObj<Bd, T: Tuple + 'static> {
    inner: Box<dyn FutureObjEndpoint<Bd, Output = T> + Send + Sync + 'static>,
}

impl<Bd, T: Tuple + 'static> EndpointObj<Bd, T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> EndpointObj<Bd, T>
    where
        E: Endpoint<Bd, Output = T> + Send + Sync + 'static,
        E::Future: Send + 'static,
    {
        EndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<Bd, T: Tuple + 'static> fmt::Debug for EndpointObj<Bd, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("EndpointObj").finish()
    }
}

impl<Bd, T: Tuple + 'static> Endpoint<Bd> for EndpointObj<Bd, T> {
    type Output = T;
    type Future = EndpointFutureObj<Bd, T>;

    #[inline]
    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
        self.inner.apply_obj(ecx)
    }
}

// ==== BoxedLocal ====
#[allow(missing_debug_implementations)]
pub struct LocalEndpointFutureObj<Bd, T>(Box<dyn EndpointFuture<Bd, Output = T> + 'static>);

impl<Bd, T> EndpointFuture<Bd> for LocalEndpointFutureObj<Bd, T> {
    type Output = T;

    #[inline]
    fn poll_endpoint(&mut self, cx: &mut Context<'_, Bd>) -> Poll<Self::Output, Error> {
        self.0.poll_endpoint(cx)
    }
}

trait LocalFutureObjEndpoint<Bd> {
    type Output: Tuple;

    fn apply_local_obj(
        &self,
        ecx: &mut ApplyContext<'_, Bd>,
    ) -> ApplyResult<LocalEndpointFutureObj<Bd, Self::Output>>;
}

impl<Bd, E: Endpoint<Bd>> LocalFutureObjEndpoint<Bd> for E
where
    E::Future: 'static,
{
    type Output = E::Output;

    #[inline(always)]
    fn apply_local_obj(
        &self,
        ecx: &mut ApplyContext<'_, Bd>,
    ) -> ApplyResult<LocalEndpointFutureObj<Bd, Self::Output>> {
        let future = self.apply(ecx)?;
        Ok(LocalEndpointFutureObj(Box::new(future)))
    }
}

#[allow(missing_docs)]
pub struct LocalEndpointObj<Bd, T: Tuple + 'static> {
    inner: Box<dyn LocalFutureObjEndpoint<Bd, Output = T> + 'static>,
}

impl<Bd, T: Tuple + 'static> LocalEndpointObj<Bd, T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> Self
    where
        E: Endpoint<Bd, Output = T> + Send + Sync + 'static,
        E::Future: 'static,
    {
        LocalEndpointObj {
            inner: Box::new(endpoint),
        }
    }
}

impl<Bd, T: Tuple + 'static> fmt::Debug for LocalEndpointObj<Bd, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("LocalEndpointObj").finish()
    }
}

impl<Bd, T: Tuple + 'static> Endpoint<Bd> for LocalEndpointObj<Bd, T> {
    type Output = T;
    type Future = LocalEndpointFutureObj<Bd, T>;

    #[inline(always)]
    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> ApplyResult<Self::Future> {
        self.inner.apply_local_obj(ecx)
    }
}
