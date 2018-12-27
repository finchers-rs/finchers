use finchers::endpoint;
use finchers::endpoint::wrapper::Wrapper;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint, IntoEndpoint};
use finchers::error::Error;

use futures::future;
use futures::{Future, Poll};

use juniper::{GraphQLType, RootNode};
use std::fmt;

use super::Schema;
use request::{GraphQLRequestEndpoint, GraphQLResponse, RequestFuture};

/// Create a GraphQL executor from the specified `RootNode`.
///
/// The endpoint created by this executor will execute the GraphQL queries
/// on the current thread.
pub fn current_thread<S>(schema: S) -> CurrentThread<S>
where
    S: Schema,
{
    CurrentThread { schema }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct CurrentThread<S> {
    schema: S,
}

impl<'a, S> IntoEndpoint<'a> for CurrentThread<S>
where
    S: Schema<Context = ()> + 'a,
{
    type Output = (GraphQLResponse,);
    type Endpoint = CurrentThreadEndpoint<endpoint::Cloned<()>, S>;

    fn into_endpoint(self) -> Self::Endpoint {
        CurrentThreadEndpoint {
            context: endpoint::cloned(()),
            request: ::request::graphql_request(),
            schema: self.schema,
        }
    }
}

impl<'a, E, CtxT, S> Wrapper<'a, E> for CurrentThread<S>
where
    E: Endpoint<'a, Output = (CtxT,)>,
    S: Schema<Context = CtxT> + 'a,
    CtxT: 'a,
{
    type Output = (GraphQLResponse,);
    type Endpoint = CurrentThreadEndpoint<E, S>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        CurrentThreadEndpoint {
            context: endpoint,
            request: ::request::graphql_request(),
            schema: self.schema,
        }
    }
}

#[derive(Debug)]
pub struct CurrentThreadEndpoint<E, S> {
    context: E,
    request: GraphQLRequestEndpoint,
    schema: S,
}

impl<'a, E, S, CtxT> Endpoint<'a> for CurrentThreadEndpoint<E, S>
where
    E: Endpoint<'a, Output = (CtxT,)>,
    S: Schema<Context = CtxT> + 'a,
    CtxT: 'a,
{
    type Output = (GraphQLResponse,);
    type Future = CurrentThreadFuture<'a, E, S::Query, S::Mutation, CtxT>;

    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let context = self.context.apply(cx)?;
        let request = self.request.apply(cx)?;
        Ok(CurrentThreadFuture {
            inner: context.join(request),
            root_node: self.schema.as_root_node(),
        })
    }
}

pub struct CurrentThreadFuture<'a, E, QueryT, MutationT, CtxT>
where
    E: Endpoint<'a, Output = (CtxT,)>,
    QueryT: GraphQLType<Context = CtxT> + 'a,
    MutationT: GraphQLType<Context = CtxT> + 'a,
    CtxT: 'a,
{
    inner: future::Join<E::Future, RequestFuture<'a>>,
    root_node: &'a RootNode<'static, QueryT, MutationT>,
}

impl<'a, E, QueryT, MutationT, CtxT> fmt::Debug
    for CurrentThreadFuture<'a, E, QueryT, MutationT, CtxT>
where
    E: Endpoint<'a, Output = (CtxT,)>,
    QueryT: GraphQLType<Context = CtxT> + 'a,
    MutationT: GraphQLType<Context = CtxT> + 'a,
    CtxT: 'a,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CurrentThreadFuture").finish()
    }
}

impl<'a, E, QueryT, MutationT, CtxT> Future for CurrentThreadFuture<'a, E, QueryT, MutationT, CtxT>
where
    E: Endpoint<'a, Output = (CtxT,)>,
    QueryT: GraphQLType<Context = CtxT>,
    MutationT: GraphQLType<Context = CtxT>,
    CtxT: 'a,
{
    type Item = (GraphQLResponse,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let ((context,), (request,)) = try_ready!(self.inner.poll());
        let response = request.execute(self.root_node, &context);
        Ok((response,).into())
    }
}
