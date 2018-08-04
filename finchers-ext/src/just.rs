use finchers_core::endpoint::Context;
use finchers_core::task;
use finchers_core::Endpoint;

/// Create an endpoint which immediately returns a value of `T`.
pub fn just<T>(x: T) -> Just<T>
where
    T: Clone + Send + Sync,
{
    Just { x }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Just<T> {
    x: T,
}

impl<T> Endpoint for Just<T>
where
    T: Clone + Send + Sync,
{
    type Output = T;
    type Task = task::Ready<T>;

    fn apply(&self, _: &mut Context) -> Option<Self::Task> {
        Some(task::ready(self.x.clone()))
    }
}
