//! Endpoint for serving GraphiQL source.

use finchers::endpoint::{ApplyContext, ApplyResult, Endpoint};
use finchers::error::Error;

use futures::{Future, Poll};

use bytes::Bytes;
use http::{header, Response};
use juniper;

/// Creates an endpoint which returns a generated GraphiQL interface.
pub fn graphiql_source(endpoint_url: impl AsRef<str>) -> GraphiQLSource {
    GraphiQLSource {
        source: juniper::http::graphiql::graphiql_source(endpoint_url.as_ref()).into(),
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct GraphiQLSource {
    source: Bytes,
}

impl GraphiQLSource {
    /// Regenerate the GraphiQL interface with the specified endpoint URL.
    pub fn regenerate(&mut self, endpoint_url: impl AsRef<str>) {
        self.source = juniper::http::graphiql::graphiql_source(endpoint_url.as_ref()).into();
    }
}

impl<'a> Endpoint<'a> for GraphiQLSource {
    type Output = (Response<Bytes>,);
    type Future = GraphiQLFuture<'a>;

    fn apply(&'a self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(GraphiQLFuture(&self.source))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct GraphiQLFuture<'a>(&'a Bytes);

impl<'a> Future for GraphiQLFuture<'a> {
    type Item = (Response<Bytes>,);
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok((Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.0.clone())
            .expect("should be a valid response"),)
            .into())
    }
}
