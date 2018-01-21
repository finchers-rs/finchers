use finchers::Responder;
use finchers::http::{header, Response, StatusCode};
use std::error::Error as StdError;
use serde_json;

use error::SearchTwitterError;
use handler::SearchTwitterItem;

#[derive(Debug, Default, Clone)]
pub struct SearchTwitterResponder;

impl Responder<SearchTwitterItem, SearchTwitterError> for SearchTwitterResponder {
    fn respond_ok(&self, item: SearchTwitterItem) -> Response {
        let body = serde_json::to_vec(&item).unwrap();
        Response::new()
            .with_header(header::ContentType::json())
            .with_header(header::ContentLength(body.len() as u64))
            .with_body(body)
    }

    fn respond_err(&self, err: SearchTwitterError) -> Response {
        match err {
            SearchTwitterError::Endpoint(e) => {
                let body = (json!({ "description": e.description(), })).to_string();
                Response::new()
                    .with_status(StatusCode::BadRequest)
                    .with_header(header::ContentType::json())
                    .with_header(header::ContentLength(body.len() as u64))
                    .with_body(body)
            }
            SearchTwitterError::Twitter(e) => {
                let body = (json!({ "description": e.description(), })).to_string();
                Response::new()
                    .with_status(StatusCode::InternalServerError)
                    .with_header(header::ContentType::json())
                    .with_header(header::ContentLength(body.len() as u64))
                    .with_body(body)
            }
        }
    }
}
