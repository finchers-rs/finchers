use std::ops::{Deref, DerefMut};
use futures::{Future, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use hyper::StatusCode;
use hyper::header::ContentType;
use hyper::mime::APPLICATION_JSON;

use combinator::body::FromBody;
use errors::*;
use request::{Body, Request};
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

impl<T: 'static> FromBody for Json<T>
where
    for<'de> T: Deserialize<'de>,
{
    type Future = Box<Future<Item = Json<T>, Error = FinchersError>>;

    fn from_body(body: Body, req: &Request) -> FinchersResult<Self::Future> {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == APPLICATION_JSON => (),
            _ => return Err(FinchersErrorKind::Status(StatusCode::BadRequest).into()),
        }
        Ok(Box::new(
            body.map_err(|err| FinchersErrorKind::ServerError(Box::new(err)).into())
                .fold(
                    Vec::new(),
                    |mut body, chunk| -> Result<Vec<u8>, FinchersError> {
                        body.extend_from_slice(&chunk);
                        Ok(body)
                    },
                )
                .and_then(|body| {
                    serde_json::from_slice(&body).map_err(|_| FinchersErrorKind::Status(StatusCode::BadRequest).into())
                })
                .map(Json),
        ))
    }
}

impl<T: Serialize> Responder for Json<T> {
    type Error = serde_json::Error;

    fn respond(self) -> Result<Response, Self::Error> {
        let body = serde_json::to_string(&self.0)?;
        Ok(
            Response::new()
                .with_header(ContentType::json())
                .with_body(body),
        )
    }
}
