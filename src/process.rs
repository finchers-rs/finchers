#![allow(missing_docs)]

use std::rc::Rc;
use std::sync::Arc;
use futures::{Future, IntoFuture};

/// A trait implemented by *server-side* processes
pub trait Process<In> {
    /// The type of values *on success*
    type Item;
    /// The type of values *on failure*
    type Error;
    /// The type of value returned from `call`
    type Future: Future<Item = Self::Item, Error = Self::Error>;

    fn call(&self, input: In) -> Self::Future;
}

impl<F, In, R> Process<In> for F
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

impl<P: Process<In>, In> Process<In> for Rc<P> {
    type Item = P::Item;
    type Error = P::Error;
    type Future = P::Future;

    fn call(&self, input: In) -> Self::Future {
        (**self).call(input)
    }
}

impl<P: Process<In>, In> Process<In> for Arc<P> {
    type Item = P::Item;
    type Error = P::Error;
    type Future = P::Future;

    fn call(&self, input: In) -> Self::Future {
        (**self).call(input)
    }
}
