use futures_util::future;
use std::marker::PhantomData;
use std::mem::PinMut;

use crate::endpoint::{Endpoint, EndpointExt};
use crate::error::Error;
use crate::input::{Cursor, Input};

/// Creates an endpoint which always rejects the request with the specified error.
pub fn reject<F, E>(f: F) -> Reject<F, E>
where
    F: Fn(PinMut<'_, Input>) -> E,
    E: Into<Error>,
{
    (Reject {
        f,
        _marker: PhantomData,
    }).output::<()>()
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Reject<F, E> {
    f: F,
    _marker: PhantomData<fn() -> E>,
}

impl<F: Copy, E> Copy for Reject<F, E> {}

impl<F: Clone, E> Clone for Reject<F, E> {
    fn clone(&self) -> Self {
        Reject {
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<F, E> Endpoint for Reject<F, E>
where
    F: Fn(PinMut<'_, Input>) -> E,
    E: Into<Error>,
{
    type Output = ();
    type Future = future::Ready<Result<Self::Output, Error>>;

    fn apply<'c>(
        &self,
        input: PinMut<'_, Input>,
        mut cursor: Cursor<'c>,
    ) -> Option<(Self::Future, Cursor<'c>)> {
        cursor.by_ref().count();
        Some((future::ready(Err((self.f)(input).into())), cursor))
    }
}
