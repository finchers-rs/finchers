use crate::endpoint::{Context, EndpointBase};
use crate::future;
use crate::generic::{one, One};
use std::fmt;
use std::marker::PhantomData;

#[allow(missing_docs)]
pub fn err<T, E: Clone>(e: E) -> Err<T, E> {
    Err {
        e,
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Err<T, E> {
    e: E,
    _marker: PhantomData<fn() -> T>,
}

impl<T, E: fmt::Debug> fmt::Debug for Err<T, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Err").field("e", &self.e).finish()
    }
}

impl<T, E: Clone> EndpointBase for Err<T, E> {
    type Output = One<Result<T, E>>;
    type Future = future::Ready<Self::Output>;

    fn apply(&self, _: &mut Context) -> Option<Self::Future> {
        Some(future::ready(one(Err(self.e.clone()))))
    }
}
