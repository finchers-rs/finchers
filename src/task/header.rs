use std::marker::PhantomData;
use futures::IntoFuture;
use futures::future::FutureResult;
use http::{header, EmptyHeader, HttpError};
use super::{Task, TaskContext};

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Header<H, E>
where
    H: header::Header + Clone,
    E: From<EmptyHeader>,
{
    pub(crate) _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Task for Header<H, E>
where
    H: header::Header + Clone,
    E: From<EmptyHeader>,
{
    type Item = H;
    type Error = E;
    type Future = FutureResult<H, Result<E, HttpError>>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        match ctx.request().header().cloned() {
            Some(h) => Ok(h).into_future(),
            None => Err(Ok(EmptyHeader(H::header_name()).into())).into_future(),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct HeaderOpt<H, E> {
    pub(crate) _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Task for HeaderOpt<H, E>
where
    H: header::Header + Clone,
{
    type Item = Option<H>;
    type Error = E;
    type Future = FutureResult<Option<H>, Result<E, HttpError>>;

    fn launch(self, ctx: &mut TaskContext) -> Self::Future {
        Ok(ctx.request().header().cloned()).into_future()
    }
}
