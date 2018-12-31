use crate::endpoint::{ApplyContext, ApplyError, ApplyResult, Endpoint};
use crate::error::Error;
use crate::future::{EndpointFuture, Poll, TaskContext};

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

impl<E: Endpoint> Wrapper<E> for OrReject {
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

impl<E: Endpoint> Endpoint for OrRejectEndpoint<E> {
    type Output = E::Output;
    type Future = OrRejectFuture<E::Future>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
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

impl<F> EndpointFuture for OrRejectFuture<F>
where
    F: EndpointFuture,
{
    type Output = F::Output;

    fn poll_endpoint(&mut self, cx: &mut TaskContext<'_>) -> Poll<Self::Output, Error> {
        match self.inner {
            Ok(ref mut f) => f.poll_endpoint(cx),
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
    F: Fn(ApplyError, &mut ApplyContext<'_>) -> R,
    R: Into<Error>,
{
    OrRejectWith { f }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrRejectWith<F> {
    f: F,
}

impl<E, F, R> Wrapper<E> for OrRejectWith<F>
where
    E: Endpoint,
    F: Fn(ApplyError, &mut ApplyContext<'_>) -> R,
    R: Into<Error>,
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

impl<E, F, R> Endpoint for OrRejectWithEndpoint<E, F>
where
    E: Endpoint,
    F: Fn(ApplyError, &mut ApplyContext<'_>) -> R,
    R: Into<Error>,
{
    type Output = E::Output;
    type Future = OrRejectFuture<E::Future>;

    fn apply(&self, ecx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
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
