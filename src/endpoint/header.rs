use std::fmt;
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
pub struct Header<H, E> {
    _marker: PhantomData<fn() -> (H, E)>,
}

impl<H, E> Copy for Header<H, E> {}

impl<H, E> Clone for Header<H, E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> fmt::Debug for Header<H, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Header").finish()
    }
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
