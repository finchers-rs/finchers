#![allow(missing_docs)]

use crate::endpoint::{Context, EndpointBase};
use crate::future;

/// Create an endpoint which applies the given function to the incoming request and returns
/// an immediate value of `T`.
///
/// NOTE: The trait bound of returned value from `F` should be replaced with `IntoFuture`.
pub fn lazy<F, T>(f: F) -> Lazy<F>
where
    F: Fn(&mut Context) -> Option<T> + Send + Sync,
    T: Send,
{
    Lazy { f }
}

#[derive(Debug, Clone, Copy)]
pub struct Lazy<F> {
    f: F,
}

impl<F, T> EndpointBase for Lazy<F>
where
    F: Fn(&mut Context) -> Option<T> + Send + Sync,
    T: Send,
{
    type Output = T;
    type Future = future::Ready<T>;

    fn apply(&self, cx: &mut Context) -> Option<Self::Future> {
        (self.f)(cx).map(future::ready)
    }
}
