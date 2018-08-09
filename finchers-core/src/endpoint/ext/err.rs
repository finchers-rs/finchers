use crate::endpoint::{Context, EndpointBase};
use crate::future;

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

    fn apply(&self, _: &mut Context) -> Option<Self::Future> {
        Some(future::ready(Err(self.e.clone())))
    }
}
