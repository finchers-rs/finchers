use crate::endpoint::{Context, EndpointBase};
use crate::future;
use crate::generic::{one, One};

#[allow(missing_docs)]
pub fn some<T: Clone>(x: T) -> Some<T> {
    Some { x }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Some<T> {
    x: T,
}

impl<T: Clone> EndpointBase for Some<T> {
    type Output = One<Option<T>>;
    type Future = future::Ready<Self::Output>;

    fn apply(&self, _: &mut Context) -> Option<Self::Future> {
        Some(future::ready(one(Some(self.x.clone()))))
    }
}
