use crate::endpoint::{Context, EndpointBase};
use crate::task;

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
    type Task = task::Ready<T>;

    fn apply(&self, _: &mut Context) -> Option<Self::Task> {
        Some(task::ready(self.x.clone()))
    }
}
