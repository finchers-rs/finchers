#![allow(missing_docs)]

use std::rc::Rc;
use std::sync::Arc;
use futures::{Future, IntoFuture};

/// A trait implemented by *server-side* processes
///
/// Roughly speaking, this trait is an abstruction of functions
/// like following signature:
///
/// ```txt
/// fn(Option<Result<In, InErr>>) -> impl Future
/// ```
pub trait Process<In, InErr> {
    /// The type of values *on success*
    type Out;
    /// The type of values *on failure*
    type Err;
    /// The type of value returned from `call`
    type Future: Future<Item = Self::Out, Error = Self::Err>;

    fn call(&self, input: Option<Result<In, InErr>>) -> Self::Future;
}

impl<F, In, InErr, R> Process<In, InErr> for F
where
    F: Fn(Option<Result<In, InErr>>) -> R,
    R: IntoFuture,
{
    type Out = R::Item;
    type Err = R::Error;
    type Future = R::Future;

    fn call(&self, input: Option<Result<In, InErr>>) -> Self::Future {
        (*self)(input).into_future()
    }
}

impl<P: Process<In, InErr>, In, InErr> Process<In, InErr> for Rc<P> {
    type Out = P::Out;
    type Err = P::Err;
    type Future = P::Future;

    fn call(&self, input: Option<Result<In, InErr>>) -> Self::Future {
        (**self).call(input)
    }
}

impl<P: Process<In, InErr>, In, InErr> Process<In, InErr> for Arc<P> {
    type Out = P::Out;
    type Err = P::Err;
    type Future = P::Future;

    fn call(&self, input: Option<Result<In, InErr>>) -> Self::Future {
        (**self).call(input)
    }
}
