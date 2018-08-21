#![allow(missing_docs)]

use futures_util::try_future::TryFutureExt;
use std::boxed::PinBox;
use std::fmt;
use std::future::{FutureObj, LocalFutureObj};

use crate::common::Tuple;
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

pub trait BoxedEndpoint<'a>: 'a {
    type Output: Tuple;

    fn apply_obj(
        &'a self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<FutureObj<'a, Result<Self::Output, Error>>>;
}

impl<'e, E> BoxedEndpoint<'e> for E
where
    E: Endpoint<'e>,
    E::Future: Send,
{
    type Output = E::Output;

    fn apply_obj(
        &'e self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<FutureObj<'e, Result<Self::Output, Error>>> {
        let future = self.apply(ecx)?.into_future();
        Ok(FutureObj::new(PinBox::new(future)))
    }
}

#[allow(missing_docs)]
pub struct Boxed<T> {
    inner: Box<dyn for<'e> BoxedEndpoint<'e, Output = T> + Send + Sync + 'static>,
}

impl<T> fmt::Debug for Boxed<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Boxed").finish()
    }
}

impl<T: Tuple> Boxed<T> {
    pub(super) fn new<E>(endpoint: E) -> Boxed<T>
    where
        for<'e> E: BoxedEndpoint<'e, Output = T> + Send + Sync + 'static,
    {
        Boxed {
            inner: Box::new(endpoint),
        }
    }
}

impl<'e, T: Tuple + 'e> Endpoint<'e> for Boxed<T> {
    type Output = T;
    type Future = FutureObj<'e, Result<T, Error>>;

    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.inner.apply_obj(ecx)
    }
}

// ==== BoxedLocal ====

pub trait LocalBoxedEndpoint<'a>: 'a {
    type Output: Tuple;

    fn apply_obj(
        &'a self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<LocalFutureObj<'a, Result<Self::Output, Error>>>;
}

impl<'e, E> LocalBoxedEndpoint<'e> for E
where
    E: Endpoint<'e>,
{
    type Output = E::Output;

    fn apply_obj(
        &'e self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<LocalFutureObj<'e, Result<Self::Output, Error>>> {
        let future = self.apply(ecx)?.into_future();
        Ok(LocalFutureObj::new(PinBox::new(future)))
    }
}

pub struct BoxedLocal<T> {
    inner: Box<dyn for<'e> LocalBoxedEndpoint<'e, Output = T> + 'static>,
}

impl<T> fmt::Debug for BoxedLocal<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("BoxedLocal").finish()
    }
}

impl<T: Tuple> BoxedLocal<T> {
    pub(super) fn new<E>(endpoint: E) -> BoxedLocal<T>
    where
        for<'e> E: LocalBoxedEndpoint<'e, Output = T> + 'static,
    {
        BoxedLocal {
            inner: Box::new(endpoint),
        }
    }
}

impl<'e, T: Tuple + 'e> Endpoint<'e> for BoxedLocal<T> {
    type Output = T;
    type Future = LocalFutureObj<'e, Result<T, Error>>;

    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.inner.apply_obj(ecx)
    }
}
