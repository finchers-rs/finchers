use std::borrow::Cow;
use std::sync::Arc;
use futures::{Future, IntoFuture};
use futures::future::FutureResult;
use hyper::Request;
use url::form_urlencoded;

pub trait Endpoint {
    type Future: Future;
    fn apply(&self, req: Arc<Request>) -> Option<Self::Future>;
}

impl<F, R> Endpoint for F
where
    F: Fn(Arc<Request>) -> Option<R>,
    R: IntoFuture,
{
    type Future = R::Future;
    fn apply(&self, req: Arc<Request>) -> Option<Self::Future> {
        (*self)(req).map(IntoFuture::into_future)
    }
}


pub struct Param {
    name: Cow<'static, str>,
}

impl Endpoint for Param {
    type Future = FutureResult<String, ParamIsNotSet>;

    fn apply(&self, req: Arc<Request>) -> Option<Self::Future> {
        Some(
            req.query()
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
