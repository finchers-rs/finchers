use futures_util::future;
use std::mem::PinMut;

use endpoint::Endpoint;
use error::Never;
use generic::Tuple;
use input::{Cursor, Input};

#[allow(missing_docs)]
pub fn ok<T: Tuple + Clone>(x: T) -> Ok<T> {
    Ok { x }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Ok<T> {
    x: T,
}

impl<T: Tuple + Clone> Endpoint for Ok<T> {
    type Ok = T;
    type Error = Never;
    type Future = future::Ready<Result<Self::Ok, Self::Error>>;

    fn apply(&self, _: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        Some((future::ready(Ok(self.x.clone())), cursor))
    }
}
