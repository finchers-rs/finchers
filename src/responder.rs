//! `Responder` layer

use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use hyper::Body;
use errors::HttpError;
use hyper::{header, Response, StatusCode};
use http::Response as HttpResponse;

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

#[allow(missing_docs)]
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    #[inline]
    fn into_response(self) -> Response {
        self
    }
}

impl<B: Into<Body>> IntoResponse for HttpResponse<B> {
    #[inline]
    fn into_response(self) -> Response {
        let (parts, body) = self.into_parts();
        HttpResponse::from_parts(parts, body.into()).into()
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::new()
            .with_status(StatusCode::NoContent)
            .with_header(header::ContentLength(0))
    }
}

impl<T: IntoResponse> IntoResponse for Option<T> {
    fn into_response(self) -> Response {
        self.map(IntoResponse::into_response).unwrap_or_else(|| {
            Response::new()
                .with_status(StatusCode::NotFound)
                .with_header(header::ContentLength(0))
        })
    }
}

impl<T: IntoResponse, E: IntoResponse> IntoResponse for Result<T, E> {
    fn into_response(self) -> Response {
        match self {
            Ok(t) => t.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.len() as u64))
            .with_body(self)
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.len() as u64))
            .with_body(self)
    }
}

impl IntoResponse for ::std::borrow::Cow<'static, str> {
    fn into_response(self) -> Response {
        Response::new()
            .with_header(header::ContentType::plaintext())
            .with_header(header::ContentLength(self.len() as u64))
            .with_body(self)
    }
}
