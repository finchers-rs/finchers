use std::ops::{Deref, DerefMut};
use hyper::header::{ContentLength, ContentType};
use hyper::mime;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

use request::{FromBody, Request};
use response::{Responder, Response};


/// Represents a JSON value
#[derive(Debug)]
pub struct Json<T = Value>(pub T);

impl<T: Serialize> Json<T> {
    #[allow(missing_docs)]
    pub fn into_value(self) -> Json<Value> {
        Json(serde_json::to_value(self.0).unwrap())
    }
}

impl<T> Deref for Json<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> FromBody for Json<T>
where
    for<'de> T: Deserialize<'de>,
{
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
        let body = serde_json::to_string(&self.0)?;
        Ok(
            Response::new()
                .with_header(ContentType::json())
                .with_header(ContentLength(body.as_bytes().len() as u64))
                .with_body(body),
        )
    }
}
