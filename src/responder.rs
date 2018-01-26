//! `Responder` layer

use std::rc::Rc;
use std::sync::Arc;
use http::{IntoResponse, Response, StatusCode};
use hyper::header;

#[allow(missing_docs)]
pub trait Responder<T, E> {
    fn respond_ok(&self, T) -> Response;

    fn respond_err(&self, E) -> Response;

    fn respond_noroute(&self) -> Response {
        Response::new().with_status(StatusCode::NotFound)
    }

    fn after_respond(&self, &mut Response) {}
}

impl<R, T, E> Responder<T, E> for Rc<R>
where
    R: Responder<T, E>,
{
    fn respond_ok(&self, input: T) -> Response {
        (**self).respond_ok(input)
    }

    fn respond_err(&self, err: E) -> Response {
        (**self).respond_err(err)
    }

    fn respond_noroute(&self) -> Response {
        (**self).respond_noroute()
    }

    fn after_respond(&self, response: &mut Response) {
        (**self).after_respond(response)
    }
}

impl<R, T, E> Responder<T, E> for Arc<R>
where
    R: Responder<T, E>,
{
    fn respond_ok(&self, input: T) -> Response {
        (**self).respond_ok(input)
    }

    fn respond_err(&self, err: E) -> Response {
        (**self).respond_err(err)
    }

    fn respond_noroute(&self) -> Response {
        (**self).respond_noroute()
    }

    fn after_respond(&self, response: &mut Response) {
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
    fn respond_ok(&self, input: T) -> Response {
        input.into_response()
    }

    fn respond_err(&self, err: E) -> Response {
        err.into_response()
    }

    fn respond_noroute(&self) -> Response {
        Response::new().with_status(StatusCode::NotFound)
    }

    fn after_respond(&self, response: &mut Response) {
        if !response.headers().has::<header::Server>() {
            response.headers_mut().set(header::Server::new("Finchers"));
        }
    }
}
