#![allow(missing_docs)]

use futures_util::try_future::TryFutureExt;
use std::boxed::PinBox;
use std::fmt;
use std::future::{FutureObj, LocalFutureObj};
use std::mem::PinMut;

use crate::endpoint::{Endpoint, EndpointResult};
use crate::error::Error;
use crate::generic::Tuple;
use crate::input::{Cursor, Input};

type EndpointFn<T> = dyn for<'a, 'c> Fn(PinMut<'a, Input>, Cursor<'c>)
        -> EndpointResult<'c, FutureObj<'static, Result<T, Error>>>
    + Send
    + Sync
    + 'static;

type LocalEndpointFn<'a, T> =
    dyn for<'i, 'c> Fn(PinMut<'i, Input>, Cursor<'c>)
            -> EndpointResult<'c, LocalFutureObj<'a, Result<T, Error>>>
        + 'a;

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
            inner: Box::new(move |input, cursor| {
                let (future, cursor) = endpoint.apply(input, cursor)?;
                Ok((FutureObj::new(PinBox::new(future.into_future())), cursor))
            }),
        }
    }
}

impl<T: Tuple> Endpoint for Boxed<T> {
    type Output = T;
    type Future = FutureObj<'static, Result<T, Error>>;

    fn apply<'c>(
        &self,
        input: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> EndpointResult<'c, Self::Future> {
        (self.inner)(input, cursor)
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
            inner: Box::new(move |input, cursor| {
                let (future, cursor) = endpoint.apply(input, cursor)?;
                Ok((
                    LocalFutureObj::new(PinBox::new(future.into_future())),
                    cursor,
                ))
            }),
        }
    }
}

impl<'a, T: Tuple> Endpoint for BoxedLocal<'a, T> {
    type Output = T;
    type Future = LocalFutureObj<'a, Result<T, Error>>;

    fn apply<'c>(
        &self,
        input: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> EndpointResult<'c, Self::Future> {
        (self.inner)(input, cursor)
    }
}
