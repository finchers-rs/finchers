//! `Responder` layer

use std::rc::Rc;
use std::sync::Arc;
use http::{Body, IntoResponse};
use http_crate::{header, Error, Response, StatusCode};

#[allow(missing_docs)]
pub trait Responder<T, E> {
    fn respond_ok(&self, T) -> Result<Response<Body>, Error>;

    fn respond_err(&self, E) -> Result<Response<Body>, Error>;

    fn respond_noroute(&self) -> Result<Response<Body>, Error>;

    fn after_respond(&self, response: &mut Response<Body>);
}

impl<R, T, E> Responder<T, E> for Rc<R>
where
    R: Responder<T, E>,
{
    fn respond_ok(&self, input: T) -> Result<Response<Body>, Error> {
        (**self).respond_ok(input)
    }

    fn respond_err(&self, err: E) -> Result<Response<Body>, Error> {
        (**self).respond_err(err)
    }

    fn respond_noroute(&self) -> Result<Response<Body>, Error> {
        (**self).respond_noroute()
    }

    fn after_respond(&self, response: &mut Response<Body>) {
        (**self).after_respond(response)
    }
}

impl<R, T, E> Responder<T, E> for Arc<R>
where
    R: Responder<T, E>,
{
    fn respond_ok(&self, input: T) -> Result<Response<Body>, Error> {
        (**self).respond_ok(input)
    }

    fn respond_err(&self, err: E) -> Result<Response<Body>, Error> {
        (**self).respond_err(err)
    }

    fn respond_noroute(&self) -> Result<Response<Body>, Error> {
        (**self).respond_noroute()
    }

    fn after_respond(&self, response: &mut Response<Body>) {
        (**self).after_respond(response)
    }
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DefaultResponder;

impl<T, E> Responder<T, E> for DefaultResponder
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn respond_ok(&self, input: T) -> Result<Response<Body>, Error> {
        input.into_response()
    }

    fn respond_err(&self, err: E) -> Result<Response<Body>, Error> {
        err.into_response()
    }

    fn respond_noroute(&self) -> Result<Response<Body>, Error> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Default::default())
    }

    fn after_respond(&self, response: &mut Response<Body>) {
        if !response.headers().contains_key(header::SERVER) {
            response
                .headers_mut()
                .insert(header::SERVER, "Finchers".parse().unwrap());
        }
    }
}
