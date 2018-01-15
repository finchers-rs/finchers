use finchers::Handler;
use finchers::errors::StdErrorResponseBuilder;
use finchers::http::{header, IntoResponse, Response};

use futures::Future;
use egg_mode;
use tokio_core::reactor::Handle;
use serde_json;

use super::SearchParams;

#[derive(Debug, Serialize)]
pub struct HandleResult {
    pub statuses: Vec<String>,
}

impl IntoResponse for HandleResult {
    fn into_response(self) -> Response {
        let body = serde_json::to_vec(&self).unwrap();
        Response::new()
            .with_header(header::ContentType::json())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }
}

#[derive(Debug)]
pub struct HandleError(egg_mode::error::Error);

impl IntoResponse for HandleError {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::server_error(self.0).finish()
    }
}

#[derive(Debug, Clone)]
pub struct SearchTwitterHandler {
    token: egg_mode::Token,
    handle: Handle,
}

impl SearchTwitterHandler {
    pub fn new(token: egg_mode::Token, handle: &Handle) -> Self {
        SearchTwitterHandler {
            token,
            handle: handle.clone(),
        }
    }
}

impl Handler<SearchParams> for SearchTwitterHandler {
    type Item = HandleResult;
    type Error = HandleError;
    type Future = Box<Future<Item = HandleResult, Error = HandleError>>;

    fn call(&self, params: SearchParams) -> Self::Future {
        Box::new(
            egg_mode::search::search(params.keyword)
                .result_type(egg_mode::search::ResultType::Recent)
                .count(params.count)
                .call(&self.token, &self.handle)
                .map_err(HandleError)
                .map(
                    |egg_mode::Response {
                         response: search, ..
                     }| {
                        HandleResult {
                            statuses: search
                                .statuses
                                .into_iter()
                                .map(|tweet| tweet.text)
                                .collect(),
                        }
                    },
                ),
        )
    }
}
