//! Definition of endpoints to parse request headers

use std::fmt;
use std::error;
use std::marker::PhantomData;
use hyper::header;

use endpoint::{Endpoint, EndpointContext};
use task::{err, ok, TaskResult};


#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct EmptyHeader(&'static str);

impl fmt::Display for EmptyHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The header '{}' is not given", self.0)
    }
}

impl error::Error for EmptyHeader {
    fn description(&self) -> &str {
        "empty header"
    }
}


/// Create an endpoint matches the value of a request header
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
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        match ctx.request().header().cloned() {
            Some(h) => Some(ok(h)),
            None => Some(err(EmptyHeader(H::header_name()).into())),
        }
    }
}



/// Create an endpoint matches the value of a request header, which the value may not exist
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
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Option<Self::Task> {
        Some(ok(ctx.request().header().cloned()))
    }
}
