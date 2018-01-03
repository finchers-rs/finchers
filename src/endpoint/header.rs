use std::marker::PhantomData;
use endpoint::{Endpoint, EndpointContext};
use http::{header, EmptyHeader};
use task;

#[allow(missing_docs)]
pub fn header<H, E>() -> Header<H, E>
where
    H: header::Header + Clone,
    E: From<EmptyHeader>,
{
    Header {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Header<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Endpoint for Header<H, E>
where
    H: header::Header + Clone,
    E: From<EmptyHeader>,
{
    type Item = H;
    type Error = E;
    type Task = task::Header<H, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(task::Header {
            _marker: PhantomData,
        })
    }
}

#[allow(missing_docs)]
pub fn header_opt<H, E>() -> HeaderOpt<H, E>
where
    H: header::Header + Clone,
{
    HeaderOpt {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct HeaderOpt<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Endpoint for HeaderOpt<H, E>
where
    H: header::Header + Clone,
{
    type Item = Option<H>;
    type Error = E;
    type Task = task::HeaderOpt<H, E>;

    fn apply(&self, _: &mut EndpointContext) -> Option<Self::Task> {
        Some(task::HeaderOpt {
            _marker: PhantomData,
        })
    }
}
