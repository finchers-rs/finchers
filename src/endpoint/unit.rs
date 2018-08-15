use futures_util::future;
use std::mem::PinMut;

use crate::endpoint::Endpoint;
use crate::error::Error;
use crate::input::{Cursor, Input};

/// Create an endpoint which simply returns an unit (`()`).
pub fn unit() -> Unit {
    Unit { _priv: () }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Unit {
    _priv: (),
}

impl Endpoint for Unit {
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(&self, _: PinMut<'_, Input>, c: Cursor<'c>) -> Option<(Self::Future, Cursor<'c>)> {
        Some((future::ready(Ok(())), c))
    }
}
