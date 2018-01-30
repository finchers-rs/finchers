//! `Responder` layer

use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use http::{header, Response, StatusCode};
use core::BodyStream;
use errors::HttpError;

#[allow(missing_docs)]
pub trait Responder {
    type Item;

    fn respond_ok(&self, item: Self::Item) -> Response<BodyStream>;

    fn respond_err(&self, &HttpError) -> Response<BodyStream>;

    fn respond_noroute(&self) -> Response<BodyStream>;

    fn after_respond(&self, response: &mut Response<BodyStream>);
}

impl<R: Responder> Responder for Rc<R> {
    type Item = R::Item;

    fn respond_ok(&self, item: Self::Item) -> Response<BodyStream> {
        (**self).respond_ok(item)
    }

    fn respond_err(&self, err: &HttpError) -> Response<BodyStream> {
        (**self).respond_err(err)
    }

    fn respond_noroute(&self) -> Response<BodyStream> {
        (**self).respond_noroute()
    }

    fn after_respond(&self, response: &mut Response<BodyStream>) {
        (**self).after_respond(response)
    }
}

impl<R: Responder> Responder for Arc<R> {
    type Item = R::Item;

    fn respond_ok(&self, item: Self::Item) -> Response<BodyStream> {
        (**self).respond_ok(item)
    }

    fn respond_err(&self, err: &HttpError) -> Response<BodyStream> {
        (**self).respond_err(err)
    }

    fn respond_noroute(&self) -> Response<BodyStream> {
        (**self).respond_noroute()
    }

    fn after_respond(&self, response: &mut Response<BodyStream>) {
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

impl<T: fmt::Display> Responder for DefaultResponder<T> {
    type Item = T;

    fn respond_ok(&self, item: Self::Item) -> Response<BodyStream> {
        let message = item.to_string();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, message.len().to_string().as_str())
            .body(::hyper::Body::from(message).into())
            .unwrap()
    }

    fn respond_err(&self, err: &HttpError) -> Response<BodyStream> {
        let message = err.to_string();
        Response::builder()
            .status(err.status_code())
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, message.len().to_string().as_str())
            .body(::hyper::Body::from(message).into())
            .unwrap()
    }

    fn respond_noroute(&self) -> Response<BodyStream> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Default::default())
            .unwrap()
    }

    fn after_respond(&self, response: &mut Response<BodyStream>) {
        if !response.headers().contains_key(header::SERVER) {
            response
                .headers_mut()
                .insert(header::SERVER, "Finchers".parse().unwrap());
        }
    }
}
