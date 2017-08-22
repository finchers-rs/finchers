use std::ops::{Deref, DerefMut};
use futures::Future;
use futures::future::AndThen;
use hyper::header::ContentType;
use hyper::mime::APPLICATION_JSON;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

use combinator::body::FromBody;
use errors::*;
use request::{Body, IntoVec, Request};
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
    type Future = AndThen<IntoVec, FinchersResult<Json<T>>, fn(Vec<u8>) -> FinchersResult<Json<T>>>;

    fn from_body(body: Body, req: &Request) -> FinchersResult<Self::Future> {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == APPLICATION_JSON => (),
            _ => bail!(FinchersErrorKind::BadRequest),
        }
        Ok(body.into_vec().and_then(|body| -> FinchersResult<_> {
            match serde_json::from_slice(&body) {
                Ok(val) => Ok(Json(val)),
                Err(_) => bail!(FinchersErrorKind::BadRequest),
            }
        }))
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
