use futures::{Future, Poll};

use crate::endpoint::{Context, Endpoint, EndpointError, EndpointResult};
use crate::error::Error;

use super::Wrapper;

/// Creates a `Wrapper` for creating an endpoint which returns the error value
/// returned from `Endpoint::apply()` as the return value from the associated `Future`.
pub fn or_reject() -> OrReject {
    OrReject { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrReject {
    _priv: (),
}

impl<'a, E: Endpoint<'a>> Wrapper<'a, E> for OrReject {
    type Output = E::Output;
    type Endpoint = OrRejectEndpoint<E>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        OrRejectEndpoint { endpoint }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OrRejectEndpoint<E> {
    endpoint: E,
}

impl<'a, E: Endpoint<'a>> Endpoint<'a> for OrRejectEndpoint<E> {
    type Output = E::Output;
    type Future = OrRejectFuture<E::Future>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match self.endpoint.apply(ecx) {
            Ok(future) => Ok(OrRejectFuture { inner: Ok(future) }),
            Err(err) => {
                while let Some(..) = ecx.next_segment() {}
                Ok(OrRejectFuture {
                    inner: Err(Some(err.into())),
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct OrRejectFuture<F> {
    inner: Result<F, Option<Error>>,
}

impl<F> Future for OrRejectFuture<F>
where
    F: Future<Error = Error>,
{
    type Item = F::Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.inner {
            Ok(ref mut f) => f.poll(),
            Err(ref mut err) => Err(err.take().unwrap()),
        }
    }
}

// ==== OrRejectWith ====

/// Creates a `Wrapper` for creating an endpoint which converts the error value
/// returned from `Endpoint::apply()` to the specified type and returns it as
/// the return value from the associated `Future`.
pub fn or_reject_with<F, R>(f: F) -> OrRejectWith<F>
where
    F: Fn(EndpointError, &mut Context<'_>) -> R,
    R: Into<Error>,
{
    OrRejectWith { f }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrRejectWith<F> {
    f: F,
}

impl<'a, E, F, R> Wrapper<'a, E> for OrRejectWith<F>
where
    E: Endpoint<'a>,
    F: Fn(EndpointError, &mut Context<'_>) -> R + 'a,
    R: Into<Error> + 'a,
{
    type Output = E::Output;
    type Endpoint = OrRejectWithEndpoint<E, F>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        OrRejectWithEndpoint {
            endpoint,
            f: self.f,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OrRejectWithEndpoint<E, F> {
    endpoint: E,
    f: F,
}

impl<'a, E, F, R> Endpoint<'a> for OrRejectWithEndpoint<E, F>
where
    E: Endpoint<'a>,
    F: Fn(EndpointError, &mut Context<'_>) -> R + 'a,
    R: Into<Error> + 'a,
{
    type Output = E::Output;
    type Future = OrRejectFuture<E::Future>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match self.endpoint.apply(ecx) {
            Ok(future) => Ok(OrRejectFuture { inner: Ok(future) }),
            Err(err) => {
                while let Some(..) = ecx.next_segment() {}
                let err = (self.f)(err, ecx).into();
                Ok(OrRejectFuture {
                    inner: Err(Some(err)),
                })
            }
        }
    }
}
