#![cfg_attr(rustfmt, rustfmt_skip)]

use finchers::Responder;
use finchers::http::{header, Response, StatusCode};
use finchers::responder::Error;
use std::error::Error as StdError;

use endpoint::EndpointError;
use handler::{SearchTwitterError, SearchTwitterItem};

macro_rules! json_response {
    (status: $status:ident,) => {{
        Response::new()
            .with_status(StatusCode::$status)
    }};
    (status: $status:ident, $($t:tt)+) => {{
        let body = json!({$($t)*}).to_string();
        Response::new()
            .with_status(StatusCode::$status)
            .with_header(header::ContentType::json())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }};
}

#[derive(Debug, Default, Clone)]
pub struct SearchTwitterResponder;

impl Responder<SearchTwitterItem, EndpointError, SearchTwitterError> for SearchTwitterResponder {
    type Response = Response;

    fn respond(&self, result: Result<SearchTwitterItem, Error<EndpointError, SearchTwitterError>>) -> Self::Response {
        match result {
            Ok(item) => {
                let body = ::serde_json::to_vec(&item).unwrap();
                Response::new()
                    .with_header(header::ContentType::json())
                    .with_header(header::ContentLength(body.len() as u64))
                    .with_body(body)
            }
            Err(Error::NoRoute) => json_response! {
                status: NotFound,
            },
            Err(Error::Endpoint(e)) => json_response! {
                status: BadRequest,
                "description": e.0.description(),
            },
            Err(Error::Handler(e)) => json_response! {
                status: InternalServerError,
                "description": e.0.description(),
            },
        }
    }
}
