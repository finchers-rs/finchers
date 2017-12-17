//! Definition of wrappers with additional check of HTTP method

use hyper::Method;

use endpoint::{Endpoint, EndpointContext, EndpointError, IntoEndpoint};

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchMethod<E: Endpoint>(Method, E);

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        let f = self.1.apply(ctx)?;
        if ctx.count_remaining_segments() > 0 {
            return Err(EndpointError::Skipped);
        }
        if *ctx.request().request().method() != self.0 {
            return Err(EndpointError::InvalidMethod);
        }
        Ok(f)
    }
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `GET`.
pub fn get<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Get, endpoint.into_endpoint())
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `POST`.
pub fn post<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Post, endpoint.into_endpoint())
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `PUT`.
pub fn put<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Put, endpoint.into_endpoint())
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `DELETE`.
pub fn delete<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Delete, endpoint.into_endpoint())
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `HEAD`.
pub fn head<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Head, endpoint.into_endpoint())
}

/// Wrap given endpoint with additional check of HTTP method,
/// successes only if its method is `PATCH`.
pub fn patch<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Patch, endpoint.into_endpoint())
}
