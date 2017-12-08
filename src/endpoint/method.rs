//! Definition of wrappers with additional check of HTTP method

use hyper::Method;

use context::Context;
use endpoint::{Endpoint, EndpointError, EndpointResult};

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchMethod<E>(Method, E);

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Future = E::Future;

    fn apply(self, ctx: &mut Context) -> EndpointResult<Self::Future> {
        let f = self.1.apply(ctx)?;
        if ctx.count_remaining_segments() > 0 {
            return Err(EndpointError::Skipped);
        }
        if *ctx.request().method() != self.0 {
            return Err(EndpointError::InvalidMethod);
        }
        Ok(f)
    }
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `GET`.
pub fn get<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Get, endpoint)
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `POST`.
pub fn post<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Post, endpoint)
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `PUT`.
pub fn put<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Put, endpoint)
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `DELETE`.
pub fn delete<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Delete, endpoint)
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `HEAD`.
pub fn head<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Head, endpoint)
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `PATCH`.
pub fn patch<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Patch, endpoint)
}
