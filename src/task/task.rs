use super::*;
use futures::{Future, IntoFuture};

pub trait Task {
    type Item;
    type Error;
    type Future: Future<Item = Self::Item, Error = Self::Error>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future;
}

impl<F: IntoFuture> Task for F {
    type Item = F::Item;
    type Error = F::Error;
    type Future = F::Future;
    fn launch(self, _: &mut TaskContext) -> Self::Future {
        self.into_future()
    }
}
