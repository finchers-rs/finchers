use std::sync::Arc;
use futures::{Future, IntoFuture};
use hyper::Request;

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
