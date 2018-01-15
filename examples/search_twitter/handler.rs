use finchers::Handler;
use finchers::errors::StdErrorResponseBuilder;
use finchers::http::{header, IntoResponse, Response};

use futures::{Future, Poll};
use egg_mode::{self, search, Token};
use egg_mode::search::ResultType;
use tokio_core::reactor::Handle;
use serde_json;

#[derive(Debug)]
pub struct SearchTwitterParam {
    query: String,
    result_type: Option<ResultType>,
    count: Option<u32>,
}

mod __impl_search_params {
    use finchers::contrib::urlencoded::{FromUrlEncoded, Parse, UrlDecodeError};
    use egg_mode::search::ResultType;
    use super::SearchTwitterParam;

    impl FromUrlEncoded for SearchTwitterParam {
        fn from_urlencoded(iter: Parse) -> Result<Self, UrlDecodeError> {
            let mut query = None;
            let mut result_type = None;
            let mut count = None;
            for (key, value) in iter {
                match &*key {
                    "q" => query = Some(value.into_owned()),
                    "type" => match &*value {
                        "recent" => result_type = Some(ResultType::Recent),
                        "popular" => result_type = Some(ResultType::Popular),
                        "mixed" => result_type = Some(ResultType::Mixed),
                        s => return Err(UrlDecodeError::invalid_value("type", s.to_owned())),
                    },
                    "count" => count = Some(value.parse().map_err(|e| UrlDecodeError::other(e))?),
                    s => return Err(UrlDecodeError::invalid_key(s.to_owned())),
                }
            }
            Ok(SearchTwitterParam {
                query: query.ok_or_else(|| UrlDecodeError::missing_key("q"))?,
                result_type,
                count,
            })
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SearchTwitterItem {
    pub statuses: Vec<String>,
}

impl IntoResponse for SearchTwitterItem {
    fn into_response(self) -> Response {
        let body = serde_json::to_vec(&self).unwrap();
        Response::new()
            .with_header(header::ContentType::json())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }
}

#[derive(Debug)]
pub struct SearchTwitterError(egg_mode::error::Error);

impl IntoResponse for SearchTwitterError {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::server_error(self.0).finish()
    }
}

#[derive(Debug, Clone)]
pub struct SearchTwitterHandler {
    token: Token,
    handle: Handle,
}

impl SearchTwitterHandler {
    pub fn new(token: Token, handle: Handle) -> Self {
        SearchTwitterHandler { token, handle }
    }
}

impl Handler<SearchTwitterParam> for SearchTwitterHandler {
    type Item = SearchTwitterItem;
    type Error = SearchTwitterError;
    type Future = SearchTwitterFuture;

    fn call(&self, param: SearchTwitterParam) -> Self::Future {
        let search = search::search(param.query)
            .result_type(param.result_type.unwrap_or(ResultType::Recent))
            .count(param.count.unwrap_or(20));
        SearchTwitterFuture {
            inner: search.call(&self.token, &self.handle),
        }
    }
}

pub struct SearchTwitterFuture {
    inner: search::SearchFuture<'static>,
}

impl Future for SearchTwitterFuture {
    type Item = SearchTwitterItem;
    type Error = SearchTwitterError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let egg_mode::Response {
            response: search, ..
        } = try_ready!(self.inner.poll().map_err(SearchTwitterError));

        let statuses = search
            .statuses
            .into_iter()
            .map(|tweet| tweet.text)
            .collect();

        Ok(SearchTwitterItem { statuses }.into())
    }
}
