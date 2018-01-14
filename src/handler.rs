#![allow(missing_docs)]

use std::rc::Rc;
use std::sync::Arc;
use futures::{Future, IntoFuture};

/// A trait implemented by *server-side* processes
pub trait Handler<In> {
    /// The type of values *on success*
    type Item;
    /// The type of values *on failure*
    type Error;
    /// The type of value returned from `call`
    type Future: Future<Item = Self::Item, Error = Self::Error>;

    fn call(&self, input: In) -> Self::Future;
}

impl<F, In, R> Handler<In> for F
where
    F: Fn(In) -> R,
    R: IntoFuture,
{
    type Item = R::Item;
    type Error = R::Error;
    type Future = R::Future;

    fn call(&self, input: In) -> Self::Future {
        (*self)(input).into_future()
    }
}

impl<H, In> Handler<In> for Rc<H>
where
    H: Handler<In>,
{
    type Item = H::Item;
    type Error = H::Error;
    type Future = H::Future;

    fn call(&self, input: In) -> Self::Future {
        (**self).call(input)
    }
}

impl<H, In> Handler<In> for Arc<H>
where
    H: Handler<In>,
{
    type Item = H::Item;
    type Error = H::Error;
    type Future = H::Future;

    fn call(&self, input: In) -> Self::Future {
        (**self).call(input)
    }
}
