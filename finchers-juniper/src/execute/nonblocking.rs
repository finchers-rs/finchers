use finchers::endpoint;
use finchers::endpoint::wrapper::Wrapper;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint, IntoEndpoint};
use finchers::error::Error;
use finchers::rt;

use futures::future;
use futures::{Future, Poll};
use std::sync::Arc;

use super::shared::SharedSchema;
use request::{GraphQLRequestEndpoint, GraphQLResponse, RequestFuture};

/// Create a GraphQL executor from the specified `RootNode`.
///
/// The endpoint created by this wrapper will spawn a task which executes the GraphQL queries
/// after receiving the request, by using tokio's `DefaultExecutor`, and notify the start of
/// the blocking section by using tokio_threadpool's blocking API.
pub fn nonblocking<S>(schema: S) -> Nonblocking<S>
where
    S: SharedSchema,
{
    Nonblocking { schema }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Nonblocking<S> {
    schema: S,
}

impl<'a, S> IntoEndpoint<'a> for Nonblocking<S>
where
    S: SharedSchema<Context = ()>,
{
    type Output = (GraphQLResponse,);
    type Endpoint = NonblockingEndpoint<endpoint::Cloned<()>, S>;

    fn into_endpoint(self) -> Self::Endpoint {
        NonblockingEndpoint {
            context: endpoint::cloned(()),
            request: ::request::graphql_request(),
            schema: Arc::new(self.schema),
        }
    }
}

impl<'a, E, S> Wrapper<'a, E> for Nonblocking<S>
where
    E: Endpoint<'a, Output = (S::Context,)>,
    S: SharedSchema,
{
    type Output = (GraphQLResponse,);
    type Endpoint = NonblockingEndpoint<E, S>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        NonblockingEndpoint {
            context: endpoint,
            request: ::request::graphql_request(),
            schema: Arc::new(self.schema),
        }
    }
}

#[derive(Debug)]
pub struct NonblockingEndpoint<E, S> {
    context: E,
    request: GraphQLRequestEndpoint,
    schema: Arc<S>,
}

impl<'a, E, S> Endpoint<'a> for NonblockingEndpoint<E, S>
where
    E: Endpoint<'a, Output = (S::Context,)>,
    S: SharedSchema,
{
    type Output = (GraphQLResponse,);
    type Future = NonblockingFuture<'a, E, S>;

    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        let context = self.context.apply(cx)?;
        let request = self.request.apply(cx)?;
        Ok(NonblockingFuture {
            inner: context.join(request),
            handle: None,
            endpoint: self,
        })
    }
}

#[allow(missing_debug_implementations)]
pub struct NonblockingFuture<'a, E: Endpoint<'a>, S: 'a> {
    inner: future::Join<E::Future, RequestFuture<'a>>,
    handle: Option<rt::SpawnHandle<GraphQLResponse, Error>>,
    endpoint: &'a NonblockingEndpoint<E, S>,
}

impl<'a, E, S> Future for NonblockingFuture<'a, E, S>
where
    E: Endpoint<'a, Output = (S::Context,)>,
    S: SharedSchema,
{
    type Item = (GraphQLResponse,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match self.handle {
                Some(ref mut handle) => return handle.poll().map(|x| x.map(|response| (response,))),
                None => {
                    let ((context,), (request,)) = try_ready!(self.inner.poll());

                    trace!("spawn a GraphQL task using the default executor");
                    let schema = self.endpoint.schema.clone();
                    let future = rt::blocking_section(move || -> ::finchers::error::Result<_> {
                        Ok(request.execute(schema.as_root_node(), &context))
                    });
                    self.handle = Some(rt::spawn_with_handle(future));
                }
            }
        }
    }
}
