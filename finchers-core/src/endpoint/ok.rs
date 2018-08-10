use futures_util::future;
use std::mem::PinMut;

use crate::endpoint::EndpointBase;
use crate::error::Never;
use crate::generic::Tuple;
use crate::input::{Cursor, Input};

#[allow(missing_docs)]
pub fn ok<T: Tuple, Clone>(x: T) -> Ok<T> {
    Ok { x }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Ok<T> {
    x: T,
}

impl<T: Tuple + Clone> EndpointBase for Ok<T> {
    type Ok = T;
    type Error = Never;
    type Future = future::Ready<Result<Self::Ok, Self::Error>>;

    fn apply(&self, _: PinMut<Input>, cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        Some((future::ready(Ok(self.x.clone())), cursor))
    }
}
