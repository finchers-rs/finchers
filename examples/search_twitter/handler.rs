use finchers::Handler;

use futures::{Future, Poll};
use egg_mode::{self, search, Token};
use egg_mode::search::ResultType;
use tokio_core::reactor::Handle;

use endpoint::SearchTwitterParam;
use error::SearchTwitterError;

#[derive(Debug, Serialize)]
pub struct Status {
    username: String,
    text: String,
    created_at: String,
    retweeted: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchTwitterItem {
    pub statuses: Vec<Status>,
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
    type Result = SearchTwitterFuture;

    fn call(&self, param: SearchTwitterParam) -> Self::Result {
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
    type Item = Option<SearchTwitterItem>;
    type Error = SearchTwitterError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let egg_mode::Response {
            response: search, ..
        } = try_ready!(self.inner.poll().map_err(SearchTwitterError::Twitter));

        let statuses = search
            .statuses
            .into_iter()
            .map(|tweet| Status {
                username: tweet
                    .user
                    .map(|u| u.screen_name)
                    .unwrap_or_else(|| "<unknown>".to_string()),
                text: tweet.text,
                created_at: tweet.created_at.to_string(),
                retweeted: tweet.retweeted.unwrap_or(false),
            })
            .collect();

        Ok(Some(SearchTwitterItem { statuses }).into())
    }
}
