//! `Responder` layer

use std::rc::Rc;
use std::string::ToString;
use std::sync::Arc;
use http::{Response, StatusCode};
use http::header;
use core::{BodyStream, HttpResponse};
use errors::Error;

#[allow(missing_docs)]
#[derive(Debug)]
pub enum Outcome<T> {
    Ok(T),
    Err(Error),
    NoRoute,
}

#[allow(missing_docs)]
impl<T> Outcome<T> {
    #[inline]
    pub fn is_ok(&self) -> bool {
        match *self {
            Outcome::Ok(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        match *self {
            Outcome::Err(..) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_noroute(&self) -> bool {
        match *self {
            Outcome::NoRoute => true,
            _ => false,
        }
    }

    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            Outcome::Ok(item) => Some(item),
            _ => None,
        }
    }

    #[inline]
    pub fn err(self) -> Option<Error> {
        match self {
            Outcome::Err(err) => Some(err),
            _ => None,
        }
    }
}

#[allow(missing_docs)]
pub trait Responder<T> {
    fn respond(&self, output: Outcome<T>) -> Response<BodyStream>;
}

impl<R: Responder<T>, T> Responder<T> for Rc<R> {
    fn respond(&self, output: Outcome<T>) -> Response<BodyStream> {
        (**self).respond(output)
    }
}

impl<R: Responder<T>, T> Responder<T> for Arc<R> {
    fn respond(&self, output: Outcome<T>) -> Response<BodyStream> {
        (**self).respond(output)
    }
}

#[allow(missing_docs)]
#[derive(Copy, Clone, Default, Debug)]
pub struct DefaultResponder {
    _priv: (),
}

impl DefaultResponder {
    fn respond_ok<T>(&self, item: T) -> Response<BodyStream>
    where
        T: ToString + HttpResponse,
    {
        let body = item.to_string();
        Response::builder()
            .status(item.status_code())
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .header(header::CONTENT_LENGTH, body.len().to_string().as_str())
            .body(body.into())
            .unwrap()
    }

    fn respond_err(&self, err: &Error) -> Response<BodyStream> {
        let body = err.to_string();
        Response::builder()
            .status(err.status_code())
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
            Outcome::Ok(item) => self.respond_ok(item).map(Into::into),
            Outcome::NoRoute => self.respond_noroute().map(Into::into),
            Outcome::Err(err) => self.respond_err(&err).map(Into::into),
        }
    }
}
