use http::header::HeaderValue;
use http::{header, Request, Response, StatusCode};
use serde::Serialize;
use serde_json;
use serde_json::Value;

use super::IntoResponse;

/// An instance of `Output` representing statically typed JSON responses.
#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> From<T> for Json<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Json(inner)
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    type Body = String;

    fn into_response(self, _: &Request<()>) -> Response<Self::Body> {
        let (status, body) = match serde_json::to_string(&self.0) {
            Ok(body) => (StatusCode::OK, body),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({
                    "code": 500,
                    "message": format!("failed to construct JSON response: {}", err),
                })
                .to_string(),
            ),
        };

        let mut response = Response::new(body);
        *response.status_mut() = status;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response
    }
}

impl IntoResponse for Value {
    type Body = String;

    fn into_response(self, _: &Request<()>) -> Response<Self::Body> {
        let body = self.to_string();
        let mut response = Response::new(body);
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        response
    }
}
