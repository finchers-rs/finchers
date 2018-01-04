#![allow(missing_docs)]

use endpoint::{Endpoint, EndpointContext, IntoEndpoint};
use http::Method;

#[derive(Debug, Clone)]
pub struct MatchMethod<E: Endpoint>(Method, E);

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Error = E::Error;
    type Task = E::Task;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        if *ctx.request().method() == self.0 {
            self.1.apply(ctx)
        } else {
            None
        }
    }
}

pub fn get<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Get, endpoint.into_endpoint())
}

pub fn post<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Post, endpoint.into_endpoint())
}

pub fn put<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Put, endpoint.into_endpoint())
}

pub fn delete<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Delete, endpoint.into_endpoint())
}

pub fn head<E, A, B>(endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint<A, B>,
{
    MatchMethod(Method::Head, endpoint.into_endpoint())
}

pub fn patch<E: Endpoint>(endpoint: E) -> MatchMethod<E> {
    MatchMethod(Method::Patch, endpoint.into_endpoint())
}
