//! The declaration of `Handler` and its implementors

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use core::Outcome;

/// A trait which represents the server-side processes.
pub trait Handler<In> {
    /// The type of returned value *on success*
    type Item;

    /// Applies this handler to an input and returns a future.
    fn call(&self, input: In) -> Outcome<Self::Item>;
}

impl<F, In, T> Handler<In> for F
where
    F: Fn(In) -> Outcome<T>,
{
    type Item = T;

    fn call(&self, input: In) -> Outcome<T> {
        (*self)(input)
    }
}

impl<H, In> Handler<In> for Rc<H>
where
    H: Handler<In>,
{
    type Item = H::Item;

    fn call(&self, input: In) -> Outcome<Self::Item> {
        (**self).call(input)
    }
}

impl<H, In> Handler<In> for Arc<H>
where
    H: Handler<In>,
{
    type Item = H::Item;

    fn call(&self, input: In) -> Outcome<Self::Item> {
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

    #[inline]
    fn call(&self, input: T) -> Outcome<T> {
        Outcome::Ok(input)
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

    #[inline]
    fn call(&self, input: Option<T>) -> Outcome<T> {
        match input {
            Some(input) => Outcome::Ok(input),
            None => Outcome::NoRoute,
        }
    }
}
