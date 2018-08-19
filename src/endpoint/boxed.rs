#![allow(missing_docs)]

use futures_util::try_future::TryFutureExt;
use std::boxed::PinBox;
use std::fmt;
use std::future::{FutureObj, LocalFutureObj};

use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;
use crate::generic::Tuple;

type EndpointFn<T> = dyn Fn(&mut Context<'_>)
        -> EndpointResult<FutureObj<'static, Result<T, Error>>>
    + Send
    + Sync
    + 'static;

type LocalEndpointFn<'a, T> =
    dyn Fn(&mut Context<'_>) -> EndpointResult<LocalFutureObj<'a, Result<T, Error>>> + 'a;

#[allow(missing_docs)]
pub struct Boxed<T> {
    inner: Box<EndpointFn<T>>,
}

impl<T> fmt::Debug for Boxed<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Boxed").finish()
    }
}

impl<T: Tuple> Boxed<T> {
    pub(super) fn new<E>(endpoint: E) -> Boxed<T>
    where
        E: Endpoint<Output = T> + Send + Sync + 'static,
        E::Future: Send + 'static,
    {
        Boxed {
            inner: Box::new(move |ecx| {
                let future = endpoint.apply(ecx)?;
                Ok(FutureObj::new(PinBox::new(future.into_future())))
            }),
        }
    }
}

impl<T: Tuple> Endpoint for Boxed<T> {
    type Output = T;
    type Future = FutureObj<'static, Result<T, Error>>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (self.inner)(ecx)
    }
}

pub struct BoxedLocal<'a, T> {
    inner: Box<LocalEndpointFn<'a, T>>,
}

impl<'a, T> fmt::Debug for BoxedLocal<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("BoxedLocal").finish()
    }
}

impl<'a, T: Tuple> BoxedLocal<'a, T> {
    pub(super) fn new<E>(endpoint: E) -> BoxedLocal<'a, T>
    where
        E: Endpoint<Output = T> + 'a,
        E::Future: 'a,
    {
        BoxedLocal {
            inner: Box::new(move |ecx| {
                let future = endpoint.apply(ecx)?;
                Ok(LocalFutureObj::new(PinBox::new(future.into_future())))
            }),
        }
    }
}

impl<'a, T: Tuple> Endpoint for BoxedLocal<'a, T> {
    type Output = T;
    type Future = LocalFutureObj<'a, Result<T, Error>>;

    fn apply(&self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        (self.inner)(ecx)
    }
}
