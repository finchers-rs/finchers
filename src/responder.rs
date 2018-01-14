//! `Responder` layer

use std::rc::Rc;
use std::sync::Arc;
use hyper;

#[derive(Debug)]
pub enum Error<E, P> {
    NoRoute,
    Endpoint(E),
    Process(P),
}

pub trait Responder<T, E, P> {
    fn respond(&self, Result<T, Error<E, P>>) -> hyper::Response;
}

impl<F, T, E, P> Responder<T, E, P> for F
where
    F: Fn(Result<T, Error<E, P>>) -> hyper::Response,
{
    fn respond(&self, input: Result<T, Error<E, P>>) -> hyper::Response {
        (*self)(input)
    }
}

impl<R, T, E, P> Responder<T, E, P> for Rc<R>
where
    R: Responder<T, E, P>,
{
    fn respond(&self, input: Result<T, Error<E, P>>) -> hyper::Response {
        (**self).respond(input)
    }
}

impl<R, T, E, P> Responder<T, E, P> for Arc<R>
where
    R: Responder<T, E, P>,
{
    fn respond(&self, input: Result<T, Error<E, P>>) -> hyper::Response {
        (**self).respond(input)
    }
}
