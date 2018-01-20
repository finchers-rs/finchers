#![allow(missing_docs)]

use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use futures::IntoFuture;

/// A trait implemented by *server-side* processes
pub trait Handler<In> {
    /// The type of values *on success*
    type Item;
    /// The type of values *on failure*
    type Error;
    /// The type of value returned from `call`
    type Result: IntoFuture<Item = Option<Self::Item>, Error = Self::Error>;

    fn call(&self, input: In) -> Self::Result;
}

impl<F, In, R, T> Handler<In> for F
where
    F: Fn(In) -> R,
    R: IntoFuture<Item = Option<T>>,
{
    type Item = T;
    type Error = R::Error;
    type Result = R;

    fn call(&self, input: In) -> Self::Result {
        (*self)(input)
    }
}

impl<H, In> Handler<In> for Rc<H>
where
    H: Handler<In>,
{
    type Item = H::Item;
    type Error = H::Error;
    type Result = H::Result;

    fn call(&self, input: In) -> Self::Result {
        (**self).call(input)
    }
}

impl<H, In> Handler<In> for Arc<H>
where
    H: Handler<In>,
{
    type Item = H::Item;
    type Error = H::Error;
    type Result = H::Result;

    fn call(&self, input: In) -> Self::Result {
        (**self).call(input)
    }
}

pub struct DefaultHandler<T, E> {
    _marker: PhantomData<fn() -> (T, E)>,
}

impl<T, E> fmt::Debug for DefaultHandler<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DefaultHandler").finish()
    }
}

impl<T, E> Handler<T> for DefaultHandler<T, E> {
    type Item = T;
    type Error = E;
    type Result = Result<Option<T>, E>;

    #[inline]
    fn call(&self, input: T) -> Self::Result {
        Ok(Some(input))
    }
}
