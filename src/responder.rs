//! `Responder` layer

use std::rc::Rc;
use std::sync::Arc;
use http::{header, HttpError, Response, StatusCode};

#[allow(missing_docs)]
pub trait Responder {
    fn respond_err(&self, &HttpError) -> Response;

    fn respond_noroute(&self) -> Response {
        Response::new().with_status(StatusCode::NotFound)
    }

    fn after_respond(&self, &mut Response) {}
}

impl<R: Responder> Responder for Rc<R> {
    fn respond_err(&self, err: &HttpError) -> Response {
        (**self).respond_err(err)
    }

    fn respond_noroute(&self) -> Response {
        (**self).respond_noroute()
    }

    fn after_respond(&self, response: &mut Response) {
        (**self).after_respond(response)
    }
}

impl<R: Responder> Responder for Arc<R> {
    fn respond_err(&self, err: &HttpError) -> Response {
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

impl Responder for DefaultResponder {
    fn respond_err(&self, err: &HttpError) -> Response {
        let message = err.to_string();
        Response::new()
            .with_status(err.status_code())
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(message.len() as u64))
            .with_body(message)
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
