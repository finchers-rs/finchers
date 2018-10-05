use hyper::body::Body;

/// A type holding the instance of request body.
#[derive(Debug)]
pub struct ReqBody {
    inner: Option<Body>,
    is_upgraded: bool,
}

impl ReqBody {
    /// Create an instance of `RequestBody` from `hyper::Body`.
    #[deprecated(
        since = "0.12.3",
        note = "This method will be removed in the future version."
    )]
    #[inline]
    pub fn from_hyp(body: Body) -> ReqBody {
        ReqBody::new(body)
    }

    pub(crate) fn new(body: Body) -> ReqBody {
        ReqBody {
            inner: Some(body),
            is_upgraded: false,
        }
    }

    #[allow(missing_docs)]
    pub fn payload(&mut self) -> Option<Body> {
        self.inner.take()
    }

    #[allow(missing_docs)]
    pub fn is_gone(&self) -> bool {
        self.inner.is_none()
    }
}

#[cfg(feature = "rt")]
mod rt {
    use super::ReqBody;

    use futures::{Future, IntoFuture};
    use hyper::body::Payload;
    use hyper::upgrade::Upgraded;
    use tokio::executor::Executor;

    use rt::DefaultExecutor;

    /// The implmentation for supporting HTTP/1.1 protocol upgrade.
    ///
    /// These methods is currently unstable and disabled by default.
    /// They are available only when the feature `rt` is set.
    impl ReqBody {
        #[allow(missing_docs)]
        #[inline]
        pub fn upgrade<F, R, Exec>(&mut self, f: F)
        where
            F: FnOnce(Upgraded) -> R + Send + 'static,
            R: IntoFuture<Item = (), Error = ()>,
            R::Future: Send + 'static,
            Exec: Executor,
        {
            self.upgrade_with_executor(f, &mut DefaultExecutor::current())
        }

        #[allow(missing_docs)]
        pub fn upgrade_with_executor<F, R, Exec>(&mut self, f: F, exec: &mut Exec)
        where
            F: FnOnce(Upgraded) -> R + Send + 'static,
            R: IntoFuture<Item = (), Error = ()>,
            R::Future: Send + 'static,
            Exec: Executor,
        {
            if let Some(body) = self.inner.take() {
                self.is_upgraded = true;
                let future = body
                    .on_upgrade()
                    .map_err(|e| error!("during upgrading the protocol: {}", e))
                    .and_then(|upgraded| f(upgraded).into_future());
                exec.spawn(Box::new(future))
                    .expect("failed to spawn the upgraded task");
            }
        }

        #[allow(missing_docs)]
        pub fn is_upgraded(&self) -> bool {
            self.is_gone() && self.is_upgraded
        }

        pub(crate) fn content_length(&self) -> Option<u64> {
            self.inner.as_ref()?.content_length()
        }
    }
}
