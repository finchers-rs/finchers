#![allow(missing_docs)]

use futures_util::try_future::TryFutureExt;
use std::boxed::PinBox;
use std::fmt;
use std::future::{FutureObj, LocalFutureObj};
use std::mem::PinMut;

use endpoint::Endpoint;
use error::Error;
use generic::Tuple;
use input::{Cursor, Input};

type EndpointFn<T> =
    dyn for<'a, 'c> Fn(PinMut<'a, Input>, Cursor<'c>)
            -> Option<(FutureObj<'static, Result<T, Error>>, Cursor<'c>)>
        + Send
        + Sync
        + 'static;

type LocalEndpointFn<'a, T> =
    dyn for<'i, 'c> Fn(PinMut<'i, Input>, Cursor<'c>)
            -> Option<(LocalFutureObj<'a, Result<T, Error>>, Cursor<'c>)>
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
                Some((FutureObj::new(PinBox::new(future.into_future())), cursor))
            }),
        }
    }
}

impl<T: Tuple> Endpoint for Boxed<T> {
    type Output = T;
    type Future = FutureObj<'static, Result<T, Error>>;

    fn apply(
        &self,
        input: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        (self.inner)(input, cursor)
    }
}

pub struct BoxedLocal<'a, T> {
    inner: Box<LocalEndpointFn<'a, T>>,
}

impl<T> fmt::Debug for BoxedLocal<'_, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("BoxedLocal").finish()
    }
}

impl<T: Tuple> BoxedLocal<'a, T> {
    pub(super) fn new<E>(endpoint: E) -> BoxedLocal<'a, T>
    where
        E: Endpoint<Output = T> + 'a,
        E::Future: 'a,
    {
        BoxedLocal {
            inner: Box::new(move |input, cursor| {
                let (future, cursor) = endpoint.apply(input, cursor)?;
                Some((
                    LocalFutureObj::new(PinBox::new(future.into_future())),
                    cursor,
                ))
            }),
        }
    }
}

impl<T: Tuple> Endpoint for BoxedLocal<'a, T> {
    type Output = T;
    type Future = LocalFutureObj<'a, Result<T, Error>>;

    fn apply(
        &self,
        input: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        (self.inner)(input, cursor)
    }
}
