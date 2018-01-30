//! `Responder` layer

use std::rc::Rc;
use std::string::ToString;
use std::sync::Arc;
use http::{Response, StatusCode};
use http::header;
use core::{BodyStream, HttpResponse, Outcome};

/// A trait to represents the conversion from outcome to an HTTP response.
pub trait Responder<T> {
    /// Convert an outcome into an HTTP response
    fn respond(&self, outcome: Outcome<T>) -> Response<BodyStream>;
}

impl<F, T> Responder<T> for F
where
    F: Fn(Outcome<T>) -> Response<BodyStream>,
{
    fn respond(&self, outcome: Outcome<T>) -> Response<BodyStream> {
        (*self)(outcome)
    }
}

impl<R: Responder<T>, T> Responder<T> for Rc<R> {
    fn respond(&self, outcome: Outcome<T>) -> Response<BodyStream> {
        (**self).respond(outcome)
    }
}

impl<R: Responder<T>, T> Responder<T> for Arc<R> {
    fn respond(&self, outcome: Outcome<T>) -> Response<BodyStream> {
        (**self).respond(outcome)
    }
}

/// A pre-defined responder for creating an HTTP response by using `ToString::to_string`.
#[derive(Copy, Clone, Default, Debug)]
pub struct DefaultResponder {
    _priv: (),
}

impl DefaultResponder {
    fn respond_item<T>(&self, item: &T) -> Response<BodyStream>
    where
        T: ?Sized + ToString + HttpResponse,
    {
        let body = item.to_string();
        Response::builder()
            .status(item.status_code())
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .header(header::CONTENT_LENGTH, body.len().to_string().as_str())
            .body(body.into())
            .unwrap()
    }

    fn respond_noroute(&self) -> Response<BodyStream> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Default::default())
            .unwrap()
    }
}

impl<T> Responder<T> for DefaultResponder
where
    T: HttpResponse + ToString,
{
    fn respond(&self, output: Outcome<T>) -> Response<BodyStream> {
        match output {
            Outcome::Ok(item) => self.respond_item(&item),
            Outcome::NoRoute => self.respond_noroute(),
            Outcome::Err(err) => self.respond_item(&*err),
        }
    }
}
