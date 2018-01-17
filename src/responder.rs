//! `Responder` layer

use std::fmt;
use std::error;
use std::rc::Rc;
use std::sync::Arc;
use http::{header, IntoResponse, Response, StatusCode};

#[allow(missing_docs)]
#[derive(Debug)]
pub enum Error<E, H> {
    NoRoute,
    Endpoint(E),
    Handler(H),
}

impl<E: fmt::Display, H: fmt::Display> fmt::Display for Error<E, H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NoRoute => f.write_str("no route"),
            Error::Endpoint(ref e) => e.fmt(f),
            Error::Handler(ref e) => e.fmt(f),
        }
    }
}

impl<E: error::Error, H: error::Error> error::Error for Error<E, H> {
    fn description(&self) -> &str {
        match *self {
            Error::NoRoute => "no route",
            Error::Endpoint(ref e) => e.description(),
            Error::Handler(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::NoRoute => None,
            Error::Endpoint(ref e) => Some(e),
            Error::Handler(ref e) => Some(e),
        }
    }
}

#[allow(missing_docs)]
pub trait Responder<T, E, H> {
    type Response: IntoResponse;

    fn respond(&self, Result<T, Error<E, H>>) -> Self::Response;
}

impl<F, T, E, H, R> Responder<T, E, H> for F
where
    F: Fn(Result<T, Error<E, H>>) -> R,
    R: IntoResponse,
{
    type Response = R;

    fn respond(&self, input: Result<T, Error<E, H>>) -> Self::Response {
        (*self)(input)
    }
}

impl<R, T, E, H> Responder<T, E, H> for Rc<R>
where
    R: Responder<T, E, H>,
{
    type Response = R::Response;

    fn respond(&self, input: Result<T, Error<E, H>>) -> Self::Response {
        (**self).respond(input)
    }
}

impl<R, T, E, H> Responder<T, E, H> for Arc<R>
where
    R: Responder<T, E, H>,
{
    type Response = R::Response;

    fn respond(&self, input: Result<T, Error<E, H>>) -> Self::Response {
        (**self).respond(input)
    }
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DefaultResponder;

impl<T, E, P> Responder<T, E, P> for DefaultResponder
where
    T: IntoResponse,
    E: IntoResponse,
    P: IntoResponse,
{
    type Response = Response;

    fn respond(&self, input: Result<T, Error<E, P>>) -> Self::Response {
        let mut response = match input {
            Ok(item) => item.into_response(),
            Err(Error::NoRoute) => Response::new().with_status(StatusCode::NotFound),
            Err(Error::Endpoint(e)) => e.into_response(),
            Err(Error::Handler(e)) => e.into_response(),
        };

        if !response.headers().has::<header::Server>() {
            response.headers_mut().set(header::Server::new("Finchers"));
        }

        response
    }
}
