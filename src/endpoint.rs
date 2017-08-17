use std::borrow::Cow;
use std::sync::Arc;
use futures::{Future, IntoFuture};
use futures::future::FutureResult;
use url::form_urlencoded;

use input::Input;

pub trait Endpoint {
    type Future: Future;
    fn apply(&self, input: Arc<Input>) -> Option<Self::Future>;
}

impl<F, R> Endpoint for F
where
    F: Fn(Arc<Input>) -> Option<R>,
    R: IntoFuture,
{
    type Future = R::Future;
    fn apply(&self, input: Arc<Input>) -> Option<Self::Future> {
        (*self)(input).map(IntoFuture::into_future)
    }
}


pub struct Param {
    name: Cow<'static, str>,
}

impl Endpoint for Param {
    type Future = FutureResult<String, ParamIsNotSet>;

    fn apply(&self, input: Arc<Input>) -> Option<Self::Future> {
        Some(
            input
                .req
                .query()
                .and_then(|query| {
                    form_urlencoded::parse(query.as_bytes())
                        .find(|&(ref k, _)| k == &self.name)
                        .map(|(_, v)| v.into_owned())
                })
                .ok_or(ParamIsNotSet(self.name.clone()))
                .into_future(),
        )
    }
}

#[derive(Debug)]
pub struct ParamIsNotSet(Cow<'static, str>);

pub fn param<S: Into<Cow<'static, str>>>(name: S) -> Param {
    Param { name: name.into() }
}
