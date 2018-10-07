use futures::{Future, Poll};
use http::{Request, Response};

use super::{Middleware, Service};

pub fn map_response_body<F>(f: F) -> MapResponseBody<F> {
    MapResponseBody(f)
}

#[derive(Debug, Clone, Copy)]
pub struct MapResponseBody<F>(F);

impl<S, ReqBody, ResBodyT, ResBodyU, F> Middleware<S> for MapResponseBody<F>
where
    S: Service<Request = Request<ReqBody>, Response = Response<ResBodyT>>,
    F: FnOnce(ResBodyT) -> ResBodyU + Clone,
{
    type Request = Request<ReqBody>;
    type Response = Response<ResBodyU>;
    type Error = S::Error;
    type Service = MapResponseBodyService<S, F>;

    fn wrap(&self, inner: S) -> Self::Service {
        MapResponseBodyService {
            inner,
            f: self.0.clone(),
        }
    }
}

#[derive(Debug)]
pub struct MapResponseBodyService<S, F> {
    inner: S,
    f: F,
}

impl<S, ReqBody, ResBodyT, ResBodyU, F> Service for MapResponseBodyService<S, F>
where
    S: Service<Request = Request<ReqBody>, Response = Response<ResBodyT>>,
    F: FnOnce(ResBodyT) -> ResBodyU + Clone,
{
    type Request = Request<ReqBody>;
    type Response = Response<ResBodyU>;
    type Error = S::Error;
    type Future = MapResponseBodyServiceFuture<S::Future, F>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.inner.poll_ready()
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        MapResponseBodyServiceFuture {
            future: self.inner.call(request),
            f: Some(self.f.clone()),
        }
    }
}

#[derive(Debug)]
pub struct MapResponseBodyServiceFuture<Fut, F> {
    future: Fut,
    f: Option<F>,
}

impl<Fut, ResBodyT, ResBodyU, F> Future for MapResponseBodyServiceFuture<Fut, F>
where
    Fut: Future<Item = Response<ResBodyT>>,
    F: FnOnce(ResBodyT) -> ResBodyU,
{
    type Item = Response<ResBodyU>;
    type Error = Fut::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future.poll().map(|x| {
            x.map(|res| {
                let f = self.f.take().expect("The future has already polled");
                res.map(f)
            })
        })
    }
}
