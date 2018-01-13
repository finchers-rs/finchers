#![allow(missing_docs)]

use futures::{future, Future, IntoFuture};
use http::{HttpError, Request};

/// Abstruction of returned value from an `Endpoint`.
pub trait EndpointResult {
    /// The type *on success*.
    type Item;

    /// The type *on failure*.
    type Error;

    /// The type of value returned from `launch`.
    type Future: Future<Item = Self::Item, Error = Result<Self::Error, HttpError>>;

    /// Launches itself and construct a `Future`, and then return it.
    ///
    /// This method will be called *after* the routing is completed.
    fn into_future(self, request: &mut Request) -> Self::Future;
}

impl<F: IntoFuture> EndpointResult for F {
    type Item = F::Item;
    type Error = F::Error;
    type Future = future::MapErr<F::Future, fn(F::Error) -> Result<F::Error, HttpError>>;

    fn into_future(self, _: &mut Request) -> Self::Future {
        self.into_future().map_err(Ok)
    }
}
