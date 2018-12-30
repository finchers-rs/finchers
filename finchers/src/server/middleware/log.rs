//! An implementation of logging middleware.

use http::{Request, Response};
use std::sync::Arc;

pub use self::impl_log::{log, LogMiddleware};
pub use self::impl_stdlog::stdlog;

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

// ==== LogMiddleware ====

mod impl_log {
    use super::super::{Middleware, Service};
    use super::{Logger, Logging};

    use futures::{Async, Future, Poll};
    use http::{Request, Response};

    /// Create a logging middleware from the specified logger.
    pub fn log<L>(logger: L) -> LogMiddleware<L>
    where
        L: Logger + Clone,
    {
        LogMiddleware { logger }
    }

    #[allow(missing_docs)]
    #[derive(Debug, Clone)]
    pub struct LogMiddleware<L> {
        logger: L,
    }

    impl<S, L, ReqBody, ResBody> Middleware<S> for LogMiddleware<L>
    where
        S: Service<Request = Request<ReqBody>, Response = Response<ResBody>>,
        L: Logger + Clone,
    {
        type Request = Request<ReqBody>;
        type Response = Response<ResBody>;
        type Error = S::Error;
        type Service = LogService<S, L>;

        fn wrap(&self, inner: S) -> Self::Service {
            LogService {
                inner,
                logger: self.logger.clone(),
            }
        }
    }

    #[derive(Debug)]
    pub struct LogService<S, L> {
        inner: S,
        logger: L,
    }

    impl<S, L, ReqBody, ResBody> Service for LogService<S, L>
    where
        S: Service<Request = Request<ReqBody>, Response = Response<ResBody>>,
        L: Logger,
    {
        type Request = Request<ReqBody>;
        type Response = Response<ResBody>;
        type Error = S::Error;
        type Future = LogServiceFuture<S::Future, L::Instance>;

        fn poll_ready(&mut self) -> Poll<(), Self::Error> {
            self.inner.poll_ready()
        }

        fn call(&mut self, request: Self::Request) -> Self::Future {
            let log_session = self.logger.start(&request);
            LogServiceFuture {
                future: self.inner.call(request),
                log_session: Some(log_session),
            }
        }
    }

    #[derive(Debug)]
    pub struct LogServiceFuture<Fut, L> {
        future: Fut,
        log_session: Option<L>,
    }

    impl<Fut, L, Bd> Future for LogServiceFuture<Fut, L>
    where
        Fut: Future<Item = Response<Bd>>,
        L: Logging,
    {
        type Item = Response<Bd>;
        type Error = Fut::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            let response = futures::try_ready!(self.future.poll());
            let instance = self
                .log_session
                .take()
                .expect("The future has already polled");
            instance.finish(&response);
            Ok(Async::Ready(response))
        }
    }
}

// ==== StdLogger ====

mod impl_stdlog {
    use super::{log, LogMiddleware, Logger, Logging};

    use http::{Method, Request, Response, Uri, Version};
    use log::{logger, Level, Record};
    use std::time::Instant;

    /// Create a logging middleware which use the standard `log` crate.
    pub fn stdlog(level: Level, target: &'static str) -> LogMiddleware<StdLog> {
        log(StdLog { level, target })
    }

    #[derive(Debug, Copy, Clone)]
    pub struct StdLog {
        level: Level,
        target: &'static str,
    }

    impl Logger for StdLog {
        type Instance = Option<StdLogInstance>;

        fn start<T>(&self, request: &Request<T>) -> Self::Instance {
            if log::log_enabled!(target: self.target, self.level) {
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
                    ))
                    .level(self.level)
                    .target(self.target)
                    .build(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::endpoint;
    use crate::server;
    use log::Level;

    #[test]
    #[ignore]
    fn compiletest_stdlog() {
        server::start(endpoint::cloned("foo"))
            .with_middleware(super::stdlog(Level::Debug, "target"))
            .serve("127.0.0.1:4000")
            .unwrap();
    }
}
