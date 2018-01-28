#![allow(missing_docs)]

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use futures::IntoFuture;

use http::{Error, IntoResponse, Response};

/// A trait implemented by *server-side* processes
pub trait Handler<In> {
    /// The type of value returned from `call`
    type Result: IntoFuture<Item = Option<Response>, Error = Error>;
    fn call(&self, input: In) -> Self::Result;
}

impl<F, In, R> Handler<In> for F
where
    F: Fn(In) -> R,
    R: IntoFuture<Item = Option<Response>, Error = Error>,
{
    type Result = R;

    fn call(&self, input: In) -> Self::Result {
        (*self)(input)
    }
}

impl<H, In> Handler<In> for Rc<H>
where
    H: Handler<In>,
{
    type Result = H::Result;

    fn call(&self, input: In) -> Self::Result {
        (**self).call(input)
    }
}

impl<H, In> Handler<In> for Arc<H>
where
    H: Handler<In>,
{
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

impl<T: IntoResponse> Handler<T> for DefaultHandler {
    type Result = Result<Option<Response>, Error>;

    #[inline]
    fn call(&self, input: T) -> Self::Result {
        Ok(Some(input.into_response()))
    }
}
