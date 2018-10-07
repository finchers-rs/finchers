//! **\[unstable\]**
//! Endpoints for supporting HTTP/1.1 protocol upgrade.
//!
//! This module is currently unstable and disabled by default.
//! It is available only when the feature `rt` is set.

#![cfg(feature = "rt")]

use std::io;

use bytes::{Buf, BufMut};
use futures::{IntoFuture, Poll};
use http::header::{HeaderName, HeaderValue};
use http::response;
use http::{HttpTryFrom, Response, StatusCode};
use hyper::upgrade::Upgraded;
use tokio::io::{AsyncRead, AsyncWrite};

use endpoint::{lazy, Lazy};
use error;
use error::Error;
use output::{Output, OutputContext};

/// An asynchronous I/O representing an upgraded HTTP connection.
///
/// This type is currently implemented as a thin wrrapper of `hyper::upgrade::Upgraded`.
#[derive(Debug)]
pub struct UpgradedIo(Upgraded);

impl io::Read for UpgradedIo {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl io::Write for UpgradedIo {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl AsyncRead for UpgradedIo {
    #[inline]
    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
        self.0.prepare_uninitialized_buffer(buf)
    }

    #[inline]
    fn read_buf<B: BufMut>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        self.0.read_buf(buf)
    }
}

impl AsyncWrite for UpgradedIo {
    #[inline]
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        AsyncWrite::shutdown(&mut self.0)
    }

    #[inline]
    fn write_buf<B: Buf>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        self.0.write_buf(buf)
    }
}

/// A builder for constructing an upgraded HTTP response.
///
/// The output to be created will spawn a task when it is converted into
/// an HTTP response. The task represents the handler of upgraded protocol.
#[derive(Debug)]
pub struct Builder {
    builder: response::Builder,
}

impl Builder {
    /// Creates a new `Builder` with the specified task executor.
    pub fn new() -> Builder {
        let mut builder = response::Builder::new();
        builder.status(StatusCode::SWITCHING_PROTOCOLS);

        Builder { builder }
    }

    /// Appends a header filed which will be inserted into the response.
    pub fn header<K, V>(mut self, name: K, value: V) -> Self
    where
        HeaderName: HttpTryFrom<K>,
        HeaderValue: HttpTryFrom<V>,
    {
        self.builder.header(name, value);
        self
    }

    /// Consumes itself and creates a new `Output` from the specified function.
    pub fn finish<F, R>(self, on_upgrade: F) -> impl Output
    where
        F: FnOnce(UpgradedIo) -> R + Send + 'static,
        R: IntoFuture<Item = (), Error = ()>,
        R::Future: Send + 'static,
    {
        UpgradeOutput {
            builder: self,
            on_upgrade,
        }
    }
}

#[derive(Debug)]
struct UpgradeOutput<F> {
    builder: Builder,
    on_upgrade: F,
}

impl<F, R> Output for UpgradeOutput<F>
where
    F: FnOnce(UpgradedIo) -> R + Send + 'static,
    R: IntoFuture<Item = (), Error = ()>,
    R::Future: Send + 'static,
{
    type Body = ();
    type Error = Error;

    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        let Self {
            builder: Builder { mut builder },
            on_upgrade,
        } = self;
        cx.input()
            .body_mut()
            .upgrade(|upgraded| on_upgrade(UpgradedIo(upgraded)));
        builder.body(()).map_err(::error::fail)
    }
}

/// Create an endpoint which just returns a value of `Builder`.
pub fn builder() -> Lazy<impl Fn() -> error::Result<Builder>> {
    lazy(|| Ok(Builder::new()))
}
