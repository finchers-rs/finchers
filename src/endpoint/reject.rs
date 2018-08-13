use futures_util::future;
use std::marker::PhantomData;
use std::mem::PinMut;

use endpoint::{Endpoint, EndpointExt};
use error::Error;
use generic::Tuple;
use input::{Cursor, Input};

/// Creates an endpoint which always rejects the request with the specified error.
pub fn reject<F, T, E>(f: F) -> Reject<F, T, E>
where
    F: Fn(PinMut<'_, Input>) -> E,
    T: Tuple,
    E: Into<Error>,
{
    (Reject {
        f,
        _marker: PhantomData,
    }).output::<T>()
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

impl<F, T, E> Endpoint for Reject<F, T, E>
where
    F: Fn(PinMut<'_, Input>) -> E,
    T: Tuple,
    E: Into<Error>,
{
    type Output = T;
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply(
        &self,
        input: PinMut<'_, Input>,
        mut cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        drop(cursor.by_ref().count());
        Some((future::ready(Err((self.f)(input).into())), cursor))
    }
}
