//! Low level HTTP definitions from Hyper

mod from_body;
mod into_body;
pub(crate) mod request;

pub use hyper::{header, mime, Body, Chunk, Error, Method, Request as HyperRequest, Response, StatusCode};
pub use hyper::header::{Header, Headers};
pub use http_crate::{Request as HttpRequest, Response as HttpResponse};

pub use self::from_body::FromBody;
pub use self::into_body::IntoBody;
pub use self::request::Request;

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
