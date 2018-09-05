#![allow(missing_docs)]

use std::fmt;
use std::pin::PinBox;

use futures_core::future::{FutureObj, LocalFutureObj};
use futures_util::try_future::TryFutureExt;

use crate::common::Tuple;
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;

pub trait IntoBoxed<T>: for<'a> BoxedEndpoint<'a, Output = T> {}

impl<T, E> IntoBoxed<T> for E where for<'a> E: BoxedEndpoint<'a, Output = T> {}

pub trait BoxedEndpoint<'a>: Send + Sync + 'static {
    type Output: Tuple;

    fn apply_obj(
        &'a self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<FutureObj<'a, Result<Self::Output, Error>>>;
}

impl<'e, E> BoxedEndpoint<'e> for E
where
    E: Endpoint<'e> + Send + Sync + 'static,
    E::Future: Send,
{
    type Output = E::Output;

    fn apply_obj(
        &'e self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<FutureObj<'e, Result<Self::Output, Error>>> {
        Ok(FutureObj::new(PinBox::new(self.apply(ecx)?.into_future())))
    }
}

#[allow(missing_docs)]
pub struct Boxed<T> {
    pub(super) inner: Box<dyn IntoBoxed<T, Output = T>>,
}

impl<T> fmt::Debug for Boxed<T> {
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

pub trait IntoBoxedLocal<T>: for<'a> LocalBoxedEndpoint<'a, Output = T> {}

impl<T, E> IntoBoxedLocal<T> for E where for<'a> E: LocalBoxedEndpoint<'a, Output = T> {}

pub trait LocalBoxedEndpoint<'a>: 'static {
    type Output: Tuple;

    fn apply_obj(
        &'a self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<LocalFutureObj<'a, Result<Self::Output, Error>>>;
}

impl<'e, E> LocalBoxedEndpoint<'e> for E
where
    E: Endpoint<'e> + 'static,
{
    type Output = E::Output;

    fn apply_obj(
        &'e self,
        ecx: &mut Context<'_>,
    ) -> EndpointResult<LocalFutureObj<'e, Result<Self::Output, Error>>> {
        Ok(LocalFutureObj::new(PinBox::new(
            self.apply(ecx)?.into_future(),
        )))
    }
}

pub struct BoxedLocal<T> {
    pub(super) inner: Box<dyn IntoBoxedLocal<T, Output = T>>,
}

impl<T> fmt::Debug for BoxedLocal<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("BoxedLocal").finish()
    }
}

impl<'e, T: Tuple + 'static> Endpoint<'e> for BoxedLocal<T> {
    type Output = T;
    type Future = LocalFutureObj<'e, Result<T, Error>>;

    #[inline(always)]
    fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        self.inner.apply_obj(ecx)
    }
}
