#![allow(deprecated)]

use endpoint::{ApplyContext, ApplyResult, Endpoint};
use error::Error;

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.9",
    note = "use `endpoint::cloned(x)` instead."
)]
#[inline]
pub fn value<T: Clone>(x: T) -> Value<T> {
    (Value { x }).with_output::<(T,)>()
}

#[doc(hidden)]
#[deprecated(
    since = "0.12.0-alpha.9",
    note = "use `endpoint::cloned(x)` instead."
)]
#[derive(Debug, Copy, Clone)]
pub struct Value<T> {
    x: T,
}

impl<'a, T: Clone + 'a> Endpoint<'a> for Value<T> {
    type Output = (T,);
    type Future = ValueFuture<'a, T>;

    fn apply(&'a self, _: &mut ApplyContext<'_>) -> ApplyResult<Self::Future> {
        Ok(ValueFuture { x: &self.x })
    }
}

#[derive(Debug)]
pub struct ValueFuture<'a, T: 'a> {
    x: &'a T,
}

impl<'a, T: Clone + 'a> ::futures::Future for ValueFuture<'a, T> {
    type Item = (T,);
    type Error = Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        Ok((self.x.clone(),).into())
    }
}
