use std::ops::{Deref, DerefMut};
use futures::Future;
use futures::future::{err, ok, AndThen, Flatten, FutureResult};
use hyper::header::{ContentLength, ContentType};
use hyper::mime::APPLICATION_JSON;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

use endpoint::body::FromBody;
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

impl<T> FromBody for Json<T>
where
    for<'de> T: Deserialize<'de>,
{
    type Error = FinchersError;
    type Future = Flatten<
        FutureResult<AndThen<IntoVec, FinchersResult<Json<T>>, fn(Vec<u8>) -> FinchersResult<Json<T>>>, FinchersError>,
    >;

    fn from_body(body: Body, req: &Request) -> Self::Future {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == APPLICATION_JSON => (),
            _ => return err(FinchersErrorKind::BadRequest.into()).flatten(),
        }
        ok(body.into_vec().and_then(
            (|body| match serde_json::from_slice(&body) {
                Ok(val) => Ok(Json(val)),
                Err(_) => bail!(FinchersErrorKind::BadRequest),
            }) as fn(Vec<u8>) -> FinchersResult<Json<T>>,
        )).flatten()
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
