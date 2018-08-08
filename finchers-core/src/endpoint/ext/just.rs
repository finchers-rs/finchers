use crate::endpoint::{Context, EndpointBase};
use crate::future;

/// Create an endpoint which immediately returns a value of `T`.
pub fn just<T: Clone>(x: T) -> Just<T> {
    Just { x }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Just<T> {
    x: T,
}

impl<T: Clone> EndpointBase for Just<T> {
    type Output = T;
    type Future = future::Ready<T>;

    fn apply(&self, _: &mut Context) -> Option<Self::Future> {
        Some(future::ready(self.x.clone()))
    }
}
