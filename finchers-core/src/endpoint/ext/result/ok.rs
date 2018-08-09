use crate::endpoint::{Context, EndpointBase};
use crate::future;
use crate::generic::{one, One};
use std::fmt;
use std::marker::PhantomData;

#[allow(missing_docs)]
pub fn ok<T: Clone, E>(x: T) -> Ok<T, E> {
    Ok {
        x,
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Ok<T, E> {
    x: T,
    _marker: PhantomData<fn() -> E>,
}

impl<T: fmt::Debug, E> fmt::Debug for Ok<T, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Ok").field("x", &self.x).finish()
    }
}

impl<T: Clone, E> EndpointBase for Ok<T, E> {
    type Output = One<Result<T, E>>;
    type Future = future::Ready<Self::Output>;

    fn apply(&self, _: &mut Context) -> Option<Self::Future> {
        Some(future::ready(one(Ok(self.x.clone()))))
    }
}
