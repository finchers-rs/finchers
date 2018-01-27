use super::Body;
use http_crate::{header, Error, Response, StatusCode};

#[allow(missing_docs)]
pub trait IntoResponse {
    fn into_response(self) -> Result<Response<Body>, Error>;
}

impl<B: Into<Body>> IntoResponse for Response<B> {
    #[inline]
    fn into_response(self) -> Result<Response<Body>, Error> {
        Ok(self.map(Into::into))
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Result<Response<Body>, Error> {
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::default())
    }
}

impl<T: IntoResponse> IntoResponse for Option<T> {
    fn into_response(self) -> Result<Response<Body>, Error> {
        match self {
            Some(r) => r.into_response(),
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::default()),
        }
    }
}

impl<T: IntoResponse, E: IntoResponse> IntoResponse for Result<T, E> {
    fn into_response(self) -> Result<Response<Body>, Error> {
        match self {
            Ok(t) => t.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Result<Response<Body>, Error> {
        Response::builder()
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, format!("{}", self.len()).as_str())
            .body(Body::from(self))
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Result<Response<Body>, Error> {
        Response::builder()
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, format!("{}", self.len()).as_str())
            .body(Body::from(self))
    }
}

impl IntoResponse for ::std::borrow::Cow<'static, str> {
    fn into_response(self) -> Result<Response<Body>, Error> {
        Response::builder()
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, format!("{}", self.len()).as_str())
            .body(Body::from(self))
    }
}
