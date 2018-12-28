use super::{Middleware, Service};
use app::AppPayload;
use error::{Error, ErrorPayload};
use output::body::ResBody;

use futures::{Async, Future, Poll};
use http::{Request, Response};

pub fn error_handler<F, Bd>(f: F) -> ErrorHandler<F>
where
    F: Fn(Error) -> Response<Bd> + Clone,
{
    ErrorHandler { f }
}

#[derive(Debug, Clone)]
pub struct ErrorHandler<F> {
    f: F,
}

impl<S, F, ReqBody, ResBd> Middleware<S> for ErrorHandler<F>
where
    S: Service<Request = Request<ReqBody>, Response = Response<AppPayload>>,
    F: Fn(Error) -> Response<ResBd> + Clone,
    ResBd: ResBody,
{
    type Request = Request<ReqBody>;
    type Response = Response<AppPayload>;
    type Error = S::Error;
    type Service = ErrorHandlingService<S, F>;

    fn wrap(&self, input: S) -> Self::Service {
        ErrorHandlingService {
            service: input,
            f: self.f.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ErrorHandlingService<S, F> {
    service: S,
    f: F,
}

impl<S, F, ReqBody, ResBd> Service for ErrorHandlingService<S, F>
where
    S: Service<Request = Request<ReqBody>, Response = Response<AppPayload>>,
    F: Fn(Error) -> Response<ResBd> + Clone,
    ResBd: ResBody,
{
    type Request = Request<ReqBody>;
    type Response = Response<AppPayload>;
    type Error = S::Error;
    type Future = ErrorHandlingFuture<S::Future, F>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, request: Self::Request) -> Self::Future {
        ErrorHandlingFuture {
            future: self.service.call(request),
            f: self.f.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ErrorHandlingFuture<Fut, F> {
    future: Fut,
    f: F,
}

impl<Fut, F, ResBd> Future for ErrorHandlingFuture<Fut, F>
where
    Fut: Future<Item = Response<AppPayload>>,
    F: Fn(Error) -> Response<ResBd>,
    ResBd: ResBody,
{
    type Item = Response<AppPayload>;
    type Error = Fut::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let response = try_ready!(self.future.poll());
        let (parts, body) = response.into_parts();
        Ok(Async::Ready(match body.downcast::<ErrorPayload>() {
            Ok(err_payload) => {
                let err = err_payload.into_inner();
                (self.f)(err).map(AppPayload::new)
            }
            Err(payload) => Response::from_parts(parts, payload),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::error_handler;
    use endpoint;
    use http::Response;
    use serde_json;
    use server;

    #[test]
    #[ignore]
    fn compiletest_error_handler() {
        let endpoint = endpoint::cloned("message");
        server::start(endpoint)
            .with_middleware(error_handler(|err| {
                let body = serde_json::to_string_pretty(&err).unwrap();
                Response::new(body)
            }))
            .serve("127.0.0.1:4000")
            .unwrap();
    }
}
