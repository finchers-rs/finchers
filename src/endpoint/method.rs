#![allow(missing_docs)]

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use http::Method;

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

pub fn method<E, A, B>(method: Method, endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod {
        method,
        endpoint: endpoint.into_endpoint(),
    }
}

#[inline]
pub fn get<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Get, endpoint)
}

#[inline]
pub fn post<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Post, endpoint)
}

#[inline]
pub fn put<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Put, endpoint)
}

#[inline]
pub fn delete<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Delete, endpoint)
}

#[inline]
pub fn head<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Head, endpoint)
}

#[inline]
pub fn patch<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Patch, endpoint)
}

#[inline]
pub fn trace<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Trace, endpoint)
}

#[inline]
pub fn connect<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Connect, endpoint)
}

#[inline]
pub fn options<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    method(Method::Options, endpoint)
}
