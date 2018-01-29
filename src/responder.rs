//! `Responder` layer

use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use errors::HttpError;
use http::{header, IntoResponse, Response, StatusCode};

#[allow(missing_docs)]
pub trait Responder {
    type Item;

    fn respond_ok(&self, item: Self::Item) -> Response;

    fn respond_err(&self, &HttpError) -> Response;

    fn respond_noroute(&self) -> Response {
        Response::new().with_status(StatusCode::NotFound)
    }

    fn after_respond(&self, &mut Response) {}
}

impl<R: Responder> Responder for Rc<R> {
    type Item = R::Item;

    fn respond_ok(&self, item: Self::Item) -> Response {
        (**self).respond_ok(item)
    }

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
    type Item = R::Item;

    fn respond_ok(&self, item: Self::Item) -> Response {
        (**self).respond_ok(item)
    }

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
pub struct DefaultResponder<T> {
    _marker: PhantomData<fn(T) -> ()>,
}

impl<T> Copy for DefaultResponder<T> {}

impl<T> Clone for DefaultResponder<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for DefaultResponder<T> {
    #[inline]
    fn default() -> Self {
        DefaultResponder {
            _marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for DefaultResponder<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DefaultResponder").finish()
    }
}

impl<T: IntoResponse> Responder for DefaultResponder<T> {
    type Item = T;

    fn respond_ok(&self, item: Self::Item) -> Response {
        item.into_response()
    }

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
