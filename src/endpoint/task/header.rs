use std::marker::PhantomData;
use futures::IntoFuture;
use futures::future::FutureResult;
use http::{header, EmptyHeader, HttpError, Request};
use super::Task;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Header<H>
where
    H: header::Header + Clone,
{
    pub(crate) _marker: PhantomData<fn() -> H>,
}

impl<H> Task for Header<H>
where
    H: header::Header + Clone,
{
    type Item = H;
    type Error = EmptyHeader;
    type Future = FutureResult<H, Result<EmptyHeader, HttpError>>;

    fn launch(self, request: &mut Request) -> Self::Future {
        match request.header().cloned() {
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

    fn launch(self, request: &mut Request) -> Self::Future {
        Ok(request.header().cloned()).into_future()
    }
}
