#![allow(missing_docs)]

use std::rc::Rc;
use std::sync::Arc;
use futures::{Future, IntoFuture};

/// A trait implemented by *server-side* processes
pub trait Process<In> {
    /// The type of values *on success*
    type Out;
    /// The type of values *on failure*
    type Err;
    /// The type of value returned from `call`
    type Future: Future<Item = Self::Out, Error = Self::Err>;

    fn call(&self, input: Option<In>) -> Self::Future;
}

impl<F, In, R> Process<In> for F
where
    F: Fn(Option<In>) -> R,
    R: IntoFuture,
{
    type Out = R::Item;
    type Err = R::Error;
    type Future = R::Future;

    fn call(&self, input: Option<In>) -> Self::Future {
        (*self)(input).into_future()
    }
}

impl<P: Process<In>, In> Process<In> for Rc<P> {
    type Out = P::Out;
    type Err = P::Err;
    type Future = P::Future;

    fn call(&self, input: Option<In>) -> Self::Future {
        (**self).call(input)
    }
}

impl<P: Process<In>, In> Process<In> for Arc<P> {
    type Out = P::Out;
    type Err = P::Err;
    type Future = P::Future;

    fn call(&self, input: Option<In>) -> Self::Future {
        (**self).call(input)
    }
}
