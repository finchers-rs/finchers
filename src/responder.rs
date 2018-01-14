//! `Responder` layer

use std::fmt;
use std::error;
use std::rc::Rc;
use std::sync::Arc;
use hyper;

#[derive(Debug)]
pub enum Error<E, P> {
    NoRoute,
    Endpoint(E),
    Process(P),
}

impl<E: fmt::Display, P: fmt::Display> fmt::Display for Error<E, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NoRoute => f.write_str("no route"),
            Error::Endpoint(ref e) => e.fmt(f),
            Error::Process(ref e) => e.fmt(f),
        }
    }
}

impl<E: error::Error, P: error::Error> error::Error for Error<E, P> {
    fn description(&self) -> &str {
        match *self {
            Error::NoRoute => "no route",
            Error::Endpoint(ref e) => e.description(),
            Error::Process(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::NoRoute => None,
            Error::Endpoint(ref e) => Some(e),
            Error::Process(ref e) => Some(e),
        }
    }
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
