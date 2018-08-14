use futures_util::future;
use std::mem::PinMut;

use crate::endpoint::Endpoint;
use crate::error::Error;
use crate::generic::Tuple;
use crate::input::{Cursor, Input};

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
    type Output = T;
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(
        &self,
        _: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        Some((future::ready(Ok(self.x.clone())), cursor))
    }
}
