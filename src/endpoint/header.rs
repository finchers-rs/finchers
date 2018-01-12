use std::fmt;
use std::marker::PhantomData;
use endpoint::{Endpoint, EndpointContext};
use http::{header, EmptyHeader};
use task;

#[allow(missing_docs)]
pub fn header<H>() -> Header<H>
where
    H: header::Header + Clone,
{
    Header {
        _marker: PhantomData,
    }
}

#[allow(missing_docs)]
pub struct Header<H> {
    _marker: PhantomData<fn() -> H>,
}

impl<H> Copy for Header<H> {}

impl<H> Clone for Header<H> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> fmt::Debug for Header<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Header").finish()
    }
}

impl<H> Endpoint for Header<H>
where
    H: header::Header + Clone,
{
    type Item = H;
    type Error = EmptyHeader;
    type Task = task::Header<H>;

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
pub struct HeaderOpt<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Copy for HeaderOpt<H, E> {}

impl<H, E> Clone for HeaderOpt<H, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> fmt::Debug for HeaderOpt<H, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HeaderOpt").finish()
    }
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
