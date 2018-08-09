use futures_util::future;

use crate::endpoint::{Context, EndpointBase};
use crate::error::Never;
use crate::generic::Tuple;

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

    fn apply(&self, _: &mut Context) -> Option<Self::Future> {
        Some(future::ready(Ok(self.x.clone())))
    }
}
