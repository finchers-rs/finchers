use std::fmt;
use std::pin::PinBox;

use futures_core::future::{FutureObj, LocalFutureObj};
use futures_util::try_future::TryFutureExt;

use crate::common::Tuple;
use crate::endpoint::{Context, Endpoint, EndpointResult, SendEndpoint};
use crate::error::Error;

trait FutureObjEndpoint<'a>: 'a {
    type Output: Tuple;

    fn apply_obj(
        &'a self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<FutureObj<'a, Result<Self::Output, Error>>>;
}

impl<'e, E: SendEndpoint<'e>> FutureObjEndpoint<'e> for E {
    type Output = E::Output;

    #[inline(always)]
    fn apply_obj(
        &'e self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<FutureObj<'e, Result<Self::Output, Error>>> {
        let future = self.apply(ecx)?.into_future();
        Ok(FutureObj::new(PinBox::new(future)))
    }
}

#[allow(missing_docs)]
pub struct EndpointObj<T: Tuple + 'static> {
    inner: Box<dyn for<'a> FutureObjEndpoint<'a, Output = T> + Send + Sync + 'static>,
}

impl<T: Tuple + 'static> EndpointObj<T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> EndpointObj<T>
    where
        for<'a> E: SendEndpoint<'a, Output = T> + Send + Sync + 'static,
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

impl<'e, T: Tuple + 'static> Endpoint<'e> for EndpointObj<T> {
    type Output = T;
    type Future = FutureObj<'e, Result<T, Error>>;

    #[inline(always)]
    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.inner.apply_obj(ecx)
    }
}

// ==== BoxedLocal ====

trait LocalFutureObjEndpoint<'a>: 'a {
    type Output: Tuple;

    fn apply_local_obj(
        &'a self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<LocalFutureObj<'a, Result<Self::Output, Error>>>;
}

impl<'e, E: Endpoint<'e>> LocalFutureObjEndpoint<'e> for E {
    type Output = E::Output;

    #[inline(always)]
    fn apply_local_obj(
        &'e self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<LocalFutureObj<'e, Result<Self::Output, Error>>> {
        let future = self.apply(ecx)?.into_future();
        Ok(LocalFutureObj::new(PinBox::new(future)))
    }
}

#[allow(missing_docs)]
pub struct LocalEndpointObj<T: Tuple + 'static> {
    inner: Box<dyn for<'a> LocalFutureObjEndpoint<'a, Output = T> + 'static>,
}

impl<T: Tuple + 'static> LocalEndpointObj<T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> LocalEndpointObj<T>
    where
        for<'a> E: Endpoint<'a, Output = T> + Send + Sync + 'static,
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

impl<'e, T: Tuple + 'static> Endpoint<'e> for LocalEndpointObj<T> {
    type Output = T;
    type Future = LocalFutureObj<'e, Result<T, Error>>;

    #[inline(always)]
    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.inner.apply_local_obj(ecx)
    }
}
