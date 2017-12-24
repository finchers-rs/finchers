//!
//! JSON support (parsing/responder) based on serde_json
//!

extern crate serde;
extern crate serde_json;

pub use self::serde_json::{Error, Value};
use self::serde::ser::Serialize;
use self::serde::de::DeserializeOwned;

use hyper::Response;
use endpoint::body::{body, Body};
use request::{BodyError, FromBody, Request};
use response::{header, mime, Responder, ResponseBuilder};


/// Represents a JSON value
#[derive(Debug)]
pub struct Json<T = Value>(pub T);

impl<T: DeserializeOwned> FromBody for Json<T> {
    type Error = Error;

    fn check_request(req: &Request) -> bool {
        req.media_type()
            .map_or(false, |m| *m == mime::APPLICATION_JSON)
    }

    fn from_body(body: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&body).map(Json)
    }
}

impl<T: Serialize> Responder for Json<T> {
    fn respond(self) -> Response {
        let body = serde_json::to_vec(&self.0).expect(concat!(
            "cannot serialize the value of type ",
            stringify!(T)
        ));
        let len = body.len();
        ResponseBuilder::default()
            .header(header::ContentType::json())
            .header(header::ContentLength(len as u64))
            .body(body)
            .finish()
    }
}

/// Create an endpoint with parsing JSON body
pub fn json_body<T: DeserializeOwned, E>() -> Body<Json<T>, E>
where
    E: From<BodyError> + From<serde_json::Error>,
{
    body::<Json<T>, E>()
}
