//! The declaration of `Handler` and its implementors

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use futures::IntoFuture;
use errors::NeverReturn;
use http::HttpError;

/// A trait which represents the server-side processes.
pub trait Handler<In> {
    /// The type of returned value *on success*
    type Item;

    /// The type of returned value *on failure*
    type Error: HttpError;

    /// The type of value returned from `call`
    type Result: IntoFuture<Item = Option<Self::Item>, Error = Self::Error>;

    /// Applies this handler to an input and returns a future.
    fn call(&self, input: In) -> Self::Result;
}

impl<F, In, R, T> Handler<In> for F
where
    F: Fn(In) -> R,
    R: IntoFuture<Item = Option<T>>,
    R::Error: HttpError,
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

/// A predefined handler to pass the input values directly
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
    type Error = NeverReturn;
    type Result = Result<Option<T>, Self::Error>;

    #[inline]
    fn call(&self, input: T) -> Self::Result {
        Ok(Some(input))
    }
}

/// A predefined handler to pass optional values.
#[derive(Copy, Clone)]
pub struct OptionalHandler {
    _priv: (),
}

impl fmt::Debug for OptionalHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OptionalHandler").finish()
    }
}

impl Default for OptionalHandler {
    fn default() -> Self {
        OptionalHandler { _priv: () }
    }
}

impl<T> Handler<Option<T>> for OptionalHandler {
    type Item = T;
    type Error = NeverReturn;
    type Result = Result<Option<T>, Self::Error>;

    #[inline]
    fn call(&self, input: Option<T>) -> Self::Result {
        Ok(input)
    }
}
