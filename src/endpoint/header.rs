use std::marker::PhantomData;
use hyper::header;

use endpoint::{Endpoint, EndpointContext, EndpointError};
use task::{ok, TaskResult};


#[derive(Debug)]
pub struct Header<H, E>(PhantomData<fn() -> (H, E)>);

impl<H, E> Clone for Header<H, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> Copy for Header<H, E> {}

impl<H: header::Header + Clone, E> Endpoint for Header<H, E> {
    type Item = H;
    type Error = E;
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        ctx.request()
            .header()
            .cloned()
            .map(ok)
            .ok_or(EndpointError::EmptyHeader)
    }
}



#[derive(Debug)]
pub struct HeaderOpt<H, E>(PhantomData<fn() -> (H, E)>);

impl<H, E> Clone for HeaderOpt<H, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, E> Copy for HeaderOpt<H, E> {}


impl<H: header::Header + Clone, E> Endpoint for HeaderOpt<H, E> {
    type Item = Option<H>;
    type Error = E;
    type Task = TaskResult<Self::Item, Self::Error>;

    fn apply(&self, ctx: &mut EndpointContext) -> Result<Self::Task, EndpointError> {
        Ok(ok(ctx.request().header().cloned()))
    }
}



pub fn header<H: header::Header + Clone, E>() -> Header<H, E> {
    Header(PhantomData)
}


pub fn header_opt<H: header::Header + Clone, E>() -> HeaderOpt<H, E> {
    HeaderOpt(PhantomData)
}
