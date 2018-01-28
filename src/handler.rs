#![allow(missing_docs)]

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use futures::IntoFuture;
use http::Error;

/// A trait which represents the server-side processes.
pub trait Handler<In> {
    /// The type of value *on success*.
    type Item;

    /// The type of value returned from `call`
    type Result: IntoFuture<Item = Option<Self::Item>, Error = Error>;

    fn call(&self, input: In) -> Self::Result;
}

impl<F, In, R, T> Handler<In> for F
where
    F: Fn(In) -> R,
    R: IntoFuture<Item = Option<T>, Error = Error>,
{
    type Item = T;
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
    type Result = H::Result;

    fn call(&self, input: In) -> Self::Result {
        (**self).call(input)
    }
}

#[derive(Copy, Clone)]
pub struct DefaultHandler {
    _priv: (),
}

impl fmt::Debug for DefaultHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DefaultHandler").finish()
    }
}

impl Default for DefaultHandler {
    fn default() -> Self {
        DefaultHandler { _priv: () }
    }
}

impl<T> Handler<T> for DefaultHandler {
    type Item = T;
    type Result = Result<Option<T>, Error>;

    #[inline]
    fn call(&self, input: T) -> Self::Result {
        Ok(Some(input))
    }
}
