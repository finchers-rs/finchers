//! Components for checking the HTTP method

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use http::Method;

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchMethod<E: Endpoint> {
    method: Method,
    endpoint: E,
}

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Result = E::Result;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Result> {
        if *ctx.method() == self.method {
            self.endpoint.apply(ctx)
        } else {
            None
        }
    }
}

#[allow(missing_docs)]
pub fn method<E, A, B>(method: Method, endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod {
        method,
        endpoint: endpoint.into_endpoint(),
    }
}

#[allow(missing_docs)]
#[inline]
pub fn get<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Get, endpoint)
}

#[allow(missing_docs)]
#[inline]
pub fn post<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Post, endpoint)
}

#[allow(missing_docs)]
#[inline]
pub fn put<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Put, endpoint)
}

#[allow(missing_docs)]
#[inline]
pub fn delete<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Delete, endpoint)
}

#[allow(missing_docs)]
#[inline]
pub fn head<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Head, endpoint)
}

#[allow(missing_docs)]
#[inline]
pub fn patch<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Patch, endpoint)
}

#[allow(missing_docs)]
#[inline]
pub fn trace<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Trace, endpoint)
}

#[allow(missing_docs)]
#[inline]
pub fn connect<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Connect, endpoint)
}

#[allow(missing_docs)]
#[inline]
pub fn options<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Options, endpoint)
}
