use finchers::endpoint;
use finchers::endpoint::wrapper::Wrapper;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint, IntoEndpoint};
use finchers::error::Error;

use futures::future;
use futures::future::Executor;
use futures::sync::oneshot;
use futures::{Future, Poll};
use std::sync::Arc;

use super::shared::SharedSchema;
use request::{GraphQLRequest, GraphQLRequestEndpoint, GraphQLResponse, RequestFuture};

/// Create a GraphQL executor from the specified `RootNode` and task executor.
///
/// The endpoint created by this wrapper will spawn a task which executes the GraphQL queries
/// after receiving the request, by using the specified `Executor<T>`.
pub fn with_spawner<S, Sp>(schema: S, spawner: Sp) -> WithSpawner<S, Sp>
where
    S: SharedSchema,
    Sp: Executor<oneshot::Execute<GraphQLTask<S>>>,
{
    WithSpawner { schema, spawner }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct WithSpawner<S, Sp> {
    schema: S,
    spawner: Sp,
}

impl<'a, S, Sp> IntoEndpoint<'a> for WithSpawner<S, Sp>
where
    S: SharedSchema<Context = ()>,
    Sp: Executor<oneshot::Execute<GraphQLTask<S>>> + 'a,
{
    type Output = (GraphQLResponse,);
    type Endpoint = WithSpawnerEndpoint<endpoint::Cloned<()>, S, Sp>;

    fn into_endpoint(self) -> Self::Endpoint {
        WithSpawnerEndpoint {
            context: endpoint::cloned(()),
            request: ::request::graphql_request(),
            schema: Arc::new(self.schema),
            spawner: self.spawner,
        }
    }
}

impl<'a, E, S, Sp> Wrapper<'a, E> for WithSpawner<S, Sp>
where
    E: Endpoint<'a, Output = (S::Context,)>,
    S: SharedSchema,
    Sp: Executor<oneshot::Execute<GraphQLTask<S>>> + 'a,
{
    type Output = (GraphQLResponse,);
    type Endpoint = WithSpawnerEndpoint<E, S, Sp>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        WithSpawnerEndpoint {
            context: endpoint,
            request: ::request::graphql_request(),
            schema: Arc::new(self.schema),
            spawner: self.spawner,
        }
    }
}

#[derive(Debug)]
pub struct WithSpawnerEndpoint<E, S, Sp> {
    context: E,
    request: GraphQLRequestEndpoint,
    schema: Arc<S>,
    spawner: Sp,
}

impl<'a, E, S, Sp> Endpoint<'a> for WithSpawnerEndpoint<E, S, Sp>
where
    E: Endpoint<'a, Output = (S::Context,)>,
    S: SharedSchema,
    Sp: Executor<oneshot::Execute<GraphQLTask<S>>> + 'a,
{
    type Output = (GraphQLResponse,);
    type Future = WithSpawnerFuture<'a, E, S, Sp>;

    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let context = self.context.apply(cx)?;
        let request = self.request.apply(cx)?;
        Ok(WithSpawnerFuture {
            inner: context.join(request),
            handle: None,
            endpoint: self,
        })
    }
}

#[allow(missing_debug_implementations)]
pub struct WithSpawnerFuture<'a, E: Endpoint<'a>, S: 'a, Sp: 'a> {
    inner: future::Join<E::Future, RequestFuture<'a>>,
    handle: Option<oneshot::SpawnHandle<GraphQLResponse, ()>>,
    endpoint: &'a WithSpawnerEndpoint<E, S, Sp>,
}

impl<'a, E, S, Sp> Future for WithSpawnerFuture<'a, E, S, Sp>
where
    E: Endpoint<'a, Output = (S::Context,)>,
    S: SharedSchema,
    Sp: Executor<oneshot::Execute<GraphQLTask<S>>>,
{
    type Item = (GraphQLResponse,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match self.handle {
                Some(ref mut handle) => {
                    return handle
                        .poll()
                        .map(|x| x.map(|response| (response,)))
                        .map_err(|_| unreachable!());
                }
                None => {
                    let ((context,), (request,)) = try_ready!(self.inner.poll());

                    trace!("spawn a GraphQL task with the specified task executor");
                    let schema = self.endpoint.schema.clone();
                    let future = GraphQLTask {
                        request,
                        schema,
                        context,
                    };
                    let handle = oneshot::spawn(future, &self.endpoint.spawner);
                    self.handle = Some(handle);
                }
            }
        }
    }
}

// not a public API.
#[allow(missing_debug_implementations)]
pub struct GraphQLTask<S: SharedSchema> {
    request: GraphQLRequest,
    schema: Arc<S>,
    context: S::Context,
}

impl<S: SharedSchema> Future for GraphQLTask<S> {
    type Item = GraphQLResponse;
    type Error = ();

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(self
            .request
            .execute(self.schema.as_root_node(), &self.context)
            .into())
    }
}
