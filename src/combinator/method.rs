//! Definition of wrappers with additional check of HTTP method

use hyper::Method;

use context::Context;
use endpoint::Endpoint;
use errors::*;
use request::Body;

#[allow(missing_docs)]
pub struct MatchMethod<E>(Method, E);

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Future = E::Future;

    fn apply<'r>(self, ctx: Context<'r>, body: Option<Body>) -> EndpointResult<'r, Self::Future> {
        if *ctx.request.method() != self.0 {
            return Err((FinchersErrorKind::Routing.into(), body));
        }
        self.1.apply(ctx, body)
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
