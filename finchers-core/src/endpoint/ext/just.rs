use crate::endpoint::{Context, EndpointBase};
use crate::future;
use crate::generic::Tuple;

/// Create an endpoint which immediately returns a value of `T`.
pub fn just<T: Clone + Tuple>(x: T) -> Just<T> {
    Just { x }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Just<T> {
    x: T,
}

impl<T: Clone + Tuple> EndpointBase for Just<T> {
    type Output = T;
    type Future = future::Ready<T>;

    fn apply(&self, _: &mut Context) -> Option<Self::Future> {
        Some(future::ready(self.x.clone()))
    }
}
