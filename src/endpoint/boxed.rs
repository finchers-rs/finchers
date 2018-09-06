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
pub struct Boxed<T: Tuple + 'static> {
    inner: Box<dyn for<'a> FutureObjEndpoint<'a, Output = T> + Send + Sync + 'static>,
}

impl<T: Tuple + 'static> Boxed<T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> Boxed<T>
    where
        for<'a> E: SendEndpoint<'a, Output = T> + Send + Sync + 'static,
    {
        Boxed {
            inner: Box::new(endpoint),
        }
    }
}

impl<T: Tuple + 'static> fmt::Debug for Boxed<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Boxed").finish()
    }
}

impl<'e, T: Tuple + 'static> Endpoint<'e> for Boxed<T> {
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
pub struct BoxedLocal<T: Tuple + 'static> {
    inner: Box<dyn for<'a> LocalFutureObjEndpoint<'a, Output = T> + 'static>,
}

impl<T: Tuple + 'static> BoxedLocal<T> {
    #[allow(missing_docs)]
    pub fn new<E>(endpoint: E) -> BoxedLocal<T>
    where
        for<'a> E: Endpoint<'a, Output = T> + Send + Sync + 'static,
    {
        BoxedLocal {
            inner: Box::new(endpoint),
        }
    }
}

impl<T: Tuple + 'static> fmt::Debug for BoxedLocal<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("BoxedLocal").finish()
    }
}

impl<'e, T: Tuple + 'static> Endpoint<'e> for BoxedLocal<T> {
    type Output = T;
    type Future = LocalFutureObj<'e, Result<T, Error>>;

    #[inline(always)]
    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.inner.apply_local_obj(ecx)
    }
}
