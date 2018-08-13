use futures_util::future;
use std::mem::PinMut;

use endpoint::Endpoint;
use error::Error;
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
    type Output = T;
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(
        &self,
        _: PinMut<'_, Input>,
        cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        Some((future::ready(Ok(self.x.clone())), cursor))
    }
}
