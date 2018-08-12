use futures_util::future;
use std::marker::PhantomData;
use std::mem::PinMut;

use endpoint::{EndpointBase, EndpointExt};
use generic::Tuple;
use input::{Cursor, Input};

/// Creates an endpoint which always rejects the request with the specified error.
pub fn reject<F, T, E>(f: F) -> Reject<F, T, E>
where
    F: Fn(PinMut<Input>) -> E,
    T: Tuple,
{
    (Reject {
        f,
        _marker: PhantomData,
    }).ok::<T>()
    .err::<E>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Reject<F, T, E> {
    f: F,
    _marker: PhantomData<fn() -> Result<T, E>>,
}

impl<F: Copy, T, E> Copy for Reject<F, T, E> {}

impl<F: Clone, T, E> Clone for Reject<F, T, E> {
    fn clone(&self) -> Self {
        Reject {
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<F, T, E> EndpointBase for Reject<F, T, E>
where
    F: Fn(PinMut<Input>) -> E,
    T: Tuple,
{
    type Ok = T;
    type Error = E;
    type Future = future::Ready<Result<Self::Ok, Self::Error>>;

    fn apply(&self, input: PinMut<Input>, mut cursor: Cursor) -> Option<(Self::Future, Cursor)> {
        unsafe {
            cursor.consume_all_segments();
        }
        Some((future::ready(Err((self.f)(input))), cursor))
    }
}
