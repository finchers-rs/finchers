//! An implementation of logging middleware.

use super::{Middleware, Service};
use http::{Request, Response};
use log::Level;
use std::sync::Arc;

/// A trait representing a logger.
pub trait Logger {
    type Instance: Logging;

    fn start<T>(&self, request: &Request<T>) -> Self::Instance;
}

impl<L: Logger> Logger for Arc<L> {
    type Instance = L::Instance;

    fn start<T>(&self, request: &Request<T>) -> Self::Instance {
        (**self).start(request)
    }
}

/// A trait representing a log session.
pub trait Logging {
    fn finish<T>(self, response: &Response<T>);
}

impl<L: Logging> Logging for Option<L> {
    fn finish<T>(self, response: &Response<T>) {
        if let Some(instance) = self {
            instance.finish(response);
        }
    }
}

// ====

/// Create a logging middleware from the specified logger.
pub fn log<F>(f: F) -> LogMiddleware<F>
where
    F: Logger + Clone,
{
    LogMiddleware { f }
}

/// Create a logging middleware which use the standard `log` crate.
pub fn stdlog(level: Level, target: &'static str) -> LogMiddleware<impl Logger + Clone + Copy> {
    log(self::imp::StdLog { level, target })
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct LogMiddleware<F> {
    f: F,
}

impl<F, S, ReqBody, ResBody> Middleware<S> for LogMiddleware<F>
where
    S: Service<Request = Request<ReqBody>, Response = Response<ResBody>>,
    F: Logger + Clone,
{
    type Request = Request<ReqBody>;
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Service = self::imp::LogService<S, F>;

    fn wrap(&self, inner: S) -> Self::Service {
        self::imp::LogService {
            inner,
            f: self.f.clone(),
        }
    }
}

mod imp {
    use super::super::Service;
    use super::{Logger, Logging};

    use futures::{Async, Future, Poll};
    use http::{Method, Request, Response, Uri, Version};
    use log::{logger, Level, Record};
    use std::time::Instant;

    #[derive(Debug)]
    pub struct LogService<S, F> {
        pub(super) inner: S,
        pub(super) f: F,
    }

    impl<F, S, ReqBody, ResBody> Service for LogService<S, F>
    where
        S: Service<Request = Request<ReqBody>, Response = Response<ResBody>>,
        F: Logger,
    {
        type Request = Request<ReqBody>;
        type Response = Response<ResBody>;
        type Error = S::Error;
        type Future = LogServiceFuture<S::Future, F::Instance>;

        fn poll_ready(&mut self) -> Poll<(), Self::Error> {
            self.inner.poll_ready()
        }

        fn call(&mut self, request: Self::Request) -> Self::Future {
            let f = self.f.start(&request);
            LogServiceFuture {
                future: self.inner.call(request),
                f: Some(f),
            }
        }
    }

    #[derive(Debug)]
    pub struct LogServiceFuture<Fut, F> {
        future: Fut,
        f: Option<F>,
    }

    impl<Fut, F, Bd> Future for LogServiceFuture<Fut, F>
    where
        Fut: Future<Item = Response<Bd>>,
        F: Logging,
    {
        type Item = Response<Bd>;
        type Error = Fut::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            let response = try_ready!(self.future.poll());
            let instance = self.f.take().expect("The future has already polled");
            instance.finish(&response);
            Ok(Async::Ready(response))
        }
    }

    // ==== StdLogger ====

    #[derive(Debug, Copy, Clone)]
    pub struct StdLog {
        pub(super) level: Level,
        pub(super) target: &'static str,
    }

    impl Logger for StdLog {
        type Instance = Option<StdLogInstance>;

        fn start<T>(&self, request: &Request<T>) -> Self::Instance {
            if log_enabled!(target:self.target, self.level) {
                let start = Instant::now();
                Some(StdLogInstance {
                    target: self.target,
                    level: self.level,
                    method: request.method().clone(),
                    uri: request.uri().clone(),
                    version: request.version(),
                    start,
                })
            } else {
                None
            }
        }
    }

    #[derive(Debug)]
    pub struct StdLogInstance {
        target: &'static str,
        level: Level,
        method: Method,
        uri: Uri,
        version: Version,
        start: Instant,
    }

    impl Logging for StdLogInstance {
        fn finish<T>(self, response: &Response<T>) {
            logger().log(
                &Record::builder()
                    .args(format_args!(
                        "{} {} -> {} ({:?})",
                        self.method,
                        self.uri,
                        response.status(),
                        self.start.elapsed()
                    )).level(self.level)
                    .target(self.target)
                    .build(),
            );
        }
    }
}
