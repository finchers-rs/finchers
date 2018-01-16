use finchers::Handler;

use futures::{Future, Poll};
use egg_mode::{self, search, Token};
use egg_mode::search::ResultType;
use tokio_core::reactor::Handle;

use endpoint::SearchTwitterParam;

#[derive(Debug, Serialize)]
pub struct SearchTwitterItem {
    pub statuses: Vec<String>,
}

#[derive(Debug)]
pub struct SearchTwitterError(pub egg_mode::error::Error);

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
