//!
//! JSON support (parsing/responder) based on serde_json
//!

extern crate serde;
extern crate serde_json;

pub use self::serde_json::{Error, Value};
use self::serde::ser::Serialize;
use self::serde::de::DeserializeOwned;

use endpoint::body::{body, Body};
use http::{self, header, mime, FromBody, Headers, IntoBody, Request};


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

impl<T: Serialize> IntoBody for Json<T> {
    fn into_body(self, h: &mut Headers) -> http::Body {
        let body = serde_json::to_vec(&self.0).expect(concat!(
            "cannot serialize the value of type ",
            stringify!(T)
        ));
        h.set(header::ContentType::json());
        h.set(header::ContentLength(body.len() as u64));
        body.into()
    }
}

impl IntoBody for Value {
    fn into_body(self, h: &mut Headers) -> http::Body {
        let body = self.to_string();
        h.set(header::ContentType::json());
        h.set(header::ContentLength(body.len() as u64));
        body.into()
    }
}


/// Create an endpoint with parsing JSON body
pub fn json_body<T: DeserializeOwned, E>() -> Body<Json<T>, E>
where
    E: From<http::Error> + From<serde_json::Error>,
{
    body::<Json<T>, E>()
}
