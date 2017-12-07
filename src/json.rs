use hyper::{header, mime};
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{self, Value};

use request::{FromBody, Request};
use response::{Responder, Response};


/// Represents a JSON value
#[derive(Debug)]
pub struct Json<T = Value>(pub T);

impl<T: DeserializeOwned> FromBody for Json<T> {
    type Error = serde_json::Error;

    fn check_request(req: &Request) -> bool {
        req.media_type()
            .map_or(false, |m| *m == mime::APPLICATION_JSON)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&body).map(Json)
    }
}

impl<T: Serialize> Responder for Json<T> {
    type Error = serde_json::Error;

    fn respond(self) -> Result<Response, Self::Error> {
        let body = serde_json::to_vec(&self.0)?;
        let len = body.len();
        Ok(Response::new()
            .with_header(header::ContentType::json())
            .with_header(header::ContentLength(len as u64))
            .with_body(body))
    }
}
