use futures_util::future;
use std::mem::PinMut;

use crate::endpoint::EndpointBase;
use crate::input::{Cursor, Input};

#[allow(missing_docs)]
pub fn err<E: Clone>(e: E) -> Err<E> {
    Err { e }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Err<E> {
    e: E,
}

impl<E: Clone> EndpointBase for Err<E> {
    type Ok = ();
    type Error = E;
    type Future = future::Ready<Result<Self::Ok, Self::Error>>;

    fn apply(&self, _: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        Some((future::ready(Err(self.e.clone())), cursor))
    }
}
