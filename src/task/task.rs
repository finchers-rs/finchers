use super::*;
use futures::{future, Future, IntoFuture};
use http::HttpError;

/// Abstruction of a `Task`, returned from an `Endpoint`.
///
/// This trait is an generalization of `IntoFuture`,
/// extended to allow access to the instance of context
/// at construction a `Future`.
pub trait Task {
    /// The type *on success*.
    type Item;

    /// The type *on failure*.
    type Error;

    /// The type of value returned from `launch`.
    type Future: Future<Item = Self::Item, Error = Result<Self::Error, HttpError>>;

    /// Launches itself and construct a `Future`, and then return it.
    ///
    /// This method will be called *after* the routing is completed.
    fn launch(self, ctx: &mut TaskContext) -> Self::Future;
}

impl<F: IntoFuture> Task for F {
    type Item = F::Item;
    type Error = F::Error;
    type Future = future::MapErr<F::Future, fn(F::Error) -> Result<F::Error, HttpError>>;

    fn launch(self, _: &mut TaskContext) -> Self::Future {
        self.into_future().map_err(Ok)
    }
}
