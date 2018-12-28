//! Endpoint for parsing GraphQL request.

use finchers::endpoint::with_get_cx;
use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint};
use finchers::endpoints::body;
use finchers::error;
use finchers::error::Error;
use finchers::output::{Output, OutputContext};

use futures::{Future, Poll};

use juniper;
use juniper::{GraphQLType, InputValue, RootNode};

use failure::SyncFailure;
use http::Method;
use http::{header, Response, StatusCode};
use percent_encoding::percent_decode;
use serde_json;
use serde_qs;

/// Create an endpoint which parses a GraphQL request from the client.
///
/// This endpoint validates if the HTTP method is GET or POST and if the iterator over remaining
/// segments is empty, and skips if the request is not acceptable.
/// If the validation is successed, it will return a Future which behaves as follows:
///
/// * If the method is `GET`, the query in the request is parsed as a single GraphQL query.
///   If the query string is missing, it will return an error.
/// * If the method is `POST`, receives the all contents of the request body and then converts
///   it into a value of `GraphQLRequest`.
///   - When `content-type` is `application/json`, the body is parsed as a JSON object which
///     contains a GraphQL query and supplemental fields if needed.
///   - When `content-type` is `application/graphql`, the body is parsed as a single GraphQL query.
pub fn graphql_request() -> GraphQLRequestEndpoint {
    GraphQLRequestEndpoint { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct GraphQLRequestEndpoint {
    _priv: (),
}

impl<'a> Endpoint<'a> for GraphQLRequestEndpoint {
    type Output = (GraphQLRequest,);
    type Future = RequestFuture<'a>;

    fn apply(&'a self, cx: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        if cx.input().method() == Method::GET {
            Ok(RequestFuture {
                kind: RequestKind::Get,
            })
        } else {
            Ok(RequestFuture {
                kind: RequestKind::Post(body::receive_all().apply(cx)?),
            })
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RequestFuture<'a> {
    kind: RequestKind<'a>,
}

#[derive(Debug)]
enum RequestKind<'a> {
    Get,
    Post(<body::ReceiveAll as Endpoint<'a>>::Future),
}

impl<'a> Future for RequestFuture<'a> {
    type Item = (GraphQLRequest,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = match self.kind {
            RequestKind::Get => with_get_cx(|input| {
                let s = input
                    .uri()
                    .query()
                    .ok_or_else(|| error::bad_request("missing query string"))?;
                parse_query_str(s)
            }),
            RequestKind::Post(ref mut f) => {
                let (data,) = try_ready!(f.poll());
                with_get_cx(
                    |input| match input.content_type().map_err(error::bad_request)? {
                        Some(m) if *m == "application/json" => {
                            serde_json::from_slice(&*data).map_err(error::bad_request)
                        }
                        Some(m) if *m == "application/graphql" => {
                            let query =
                                String::from_utf8(data.to_vec()).map_err(error::bad_request)?;
                            Ok(GraphQLRequest::single(query, None, None))
                        }
                        Some(_m) => Err(error::bad_request("unsupported content-type.")),
                        None => Err(error::bad_request("missing content-type.")),
                    },
                )
            }
        };

        result.map(|request| (request,).into())
    }
}

// ==== GraphQLRequest ====

/// A type representing the decoded GraphQL query obtained by parsing an HTTP request.
#[derive(Debug, Deserialize)]
pub struct GraphQLRequest(GraphQLRequestKind);

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GraphQLRequestKind {
    Single(juniper::http::GraphQLRequest),
    Batch(Vec<juniper::http::GraphQLRequest>),
}

impl GraphQLRequest {
    fn single(
        query: String,
        operation_name: Option<String>,
        variables: Option<InputValue>,
    ) -> GraphQLRequest {
        GraphQLRequest(GraphQLRequestKind::Single(
            juniper::http::GraphQLRequest::new(query, operation_name, variables),
        ))
    }

    /// Executes a GraphQL query represented by this value using the specified schema and context.
    pub fn execute<QueryT, MutationT, CtxT>(
        &self,
        root_node: &RootNode<'static, QueryT, MutationT>,
        context: &CtxT,
    ) -> GraphQLResponse
    where
        QueryT: GraphQLType<Context = CtxT>,
        MutationT: GraphQLType<Context = CtxT>,
    {
        use self::GraphQLRequestKind::*;
        match self.0 {
            Single(ref request) => {
                let response = request.execute(root_node, context);
                GraphQLResponse {
                    is_ok: response.is_ok(),
                    body: serde_json::to_vec(&response),
                }
            }
            Batch(ref requests) => {
                let responses: Vec<_> = requests
                    .iter()
                    .map(|request| request.execute(root_node, context))
                    .collect();
                GraphQLResponse {
                    is_ok: responses.iter().all(|response| response.is_ok()),
                    body: serde_json::to_vec(&responses),
                }
            }
        }
    }
}

fn parse_query_str(s: &str) -> Result<GraphQLRequest, Error> {
    #[derive(Debug, Deserialize)]
    struct ParsedQuery {
        query: String,
        operation_name: Option<String>,
        variables: Option<String>,
    }

    let parsed: ParsedQuery =
        serde_qs::from_str(s).map_err(|e| error::fail(SyncFailure::new(e)))?;

    let query = percent_decode(parsed.query.as_bytes())
        .decode_utf8()
        .map_err(error::bad_request)?
        .into_owned();

    let operation_name = match parsed.operation_name {
        Some(s) => Some(
            percent_decode(s.as_bytes())
                .decode_utf8()
                .map_err(error::bad_request)?
                .into_owned(),
        ),
        None => None,
    };

    let variables: Option<InputValue> = match parsed.variables {
        Some(variables) => {
            let decoded = percent_decode(variables.as_bytes())
                .decode_utf8()
                .map_err(error::bad_request)?;
            serde_json::from_str(&*decoded)
                .map(Some)
                .map_err(error::bad_request)?
        }
        None => None,
    };

    Ok(GraphQLRequest::single(query, operation_name, variables))
}

/// A type representing the result from executing a GraphQL query.
#[derive(Debug)]
pub struct GraphQLResponse {
    is_ok: bool,
    body: Result<Vec<u8>, serde_json::Error>,
}

impl Output for GraphQLResponse {
    type Body = Vec<u8>;
    type Error = Error;

    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let status = if self.is_ok {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        let body = self.body.map_err(error::fail)?;
        Ok(Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .expect("should be a valid response"))
    }
}

#[cfg(test)]
mod tests {
    use finchers::test;
    use http::Request;

    use super::{graphql_request, GraphQLRequest, GraphQLRequestKind};

    #[test]
    fn test_get_request() {
        let mut runner = test::runner(graphql_request());
        assert_matches!(
            runner.apply(Request::get("/?query={{}}")),
            Ok(GraphQLRequest(GraphQLRequestKind::Single(..)))
        );
    }

    #[test]
    fn test_json_request() {
        let mut runner = test::runner(graphql_request());
        assert_matches!(
            runner.apply(
                Request::post("/")
                    .header("content-type", "application/json")
                    .body(r#"{ "query": "{ apiVersion }" }"#),
            ),
            Ok(GraphQLRequest(GraphQLRequestKind::Single(..)))
        );
    }

    #[test]
    fn test_batch_json_request() {
        let mut runner = test::runner(graphql_request());
        assert_matches!(
            runner.apply(
                Request::post("/")
                    .header("content-type", "application/json")
                    .body(
                        r#"[
                      { "query": "{ apiVersion }" },
                      { "query": "{ me { id } }" }
                    ]"#,
                    ),
            ),
            Ok(GraphQLRequest(GraphQLRequestKind::Batch(..)))
        );
    }

    #[test]
    fn test_graphql_request() {
        let mut runner = test::runner(graphql_request());
        assert_matches!(
            runner.apply(
                Request::post("/")
                    .header("content-type", "application/graphql")
                    .body(r#"{ apiVersion }"#),
            ),
            Ok(GraphQLRequest(GraphQLRequestKind::Single(..)))
        );
    }
}
