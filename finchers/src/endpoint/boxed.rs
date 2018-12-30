use futures::Future;
use std::fmt;

use crate::common::Tuple;
use crate::endpoint::{ApplyContext, ApplyResult, Endpoint, SendEndpoint};
use crate::error::Error;

trait FutureObjEndpoint {
    type Output: Tuple;

    fn apply_obj(
        &self,
        ecx: &mut ApplyContext<'_>,
    ) -> ApplyResult<Box<dyn Future<Item = Self::Output, Error = Error> + Send + 'static>>;
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
    ) -> ApplyResult<Box<dyn Future<Item = Self::Output, Error = Error> + Send + 'static>> {
        let future = self.apply_send(ecx)?;
        Ok(Box::new(future))
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
    type Future = Box<dyn Future<Item = Self::Output, Error = Error> + Send + 'static>;

    #[inline(always)]
    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        self.inner.apply_obj(ecx)
    }
}

// ==== BoxedLocal ====

trait LocalFutureObjEndpoint {
    type Output: Tuple;

    fn apply_local_obj(
        &self,
        ecx: &mut ApplyContext<'_>,
    ) -> ApplyResult<Box<dyn Future<Item = Self::Output, Error = Error> + 'static>>;
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
    ) -> ApplyResult<Box<dyn Future<Item = Self::Output, Error = Error> + 'static>> {
        let future = self.apply(ecx)?;
        Ok(Box::new(future))
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
    type Future = Box<dyn Future<Item = Self::Output, Error = Error> + 'static>;

    #[inline(always)]
    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        self.inner.apply_local_obj(ecx)
    }
}
