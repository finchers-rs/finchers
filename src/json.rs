use std::ops::{Deref, DerefMut};
use futures::{Future, Stream};
use serde::{Serialize, Deserialize};
use serde_json::{self, Value};
use hyper::header::ContentType;
use hyper::mime::APPLICATION_JSON;

use combinator::body::{FromBody, FromBodyFuture};
use request::{Request, Body};
use response::{Response, Responder};


#[derive(Debug)]
pub struct Json<T = Value>(pub T);

impl<T: Serialize> Json<T> {
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
    type Future = FromBodyFuture<Box<Future<Item = Json<T>, Error = ()>>>;

    fn from_body(body: Body, req: &Request) -> Self::Future {
        match req.header() {
            Some(&ContentType(ref mime)) if *mime == APPLICATION_JSON => (),
            _ => return FromBodyFuture::WrongMediaType,
        }
        FromBodyFuture::Parsed(Box::new(
            body.map_err(|_| ())
                .fold(Vec::new(), |mut body, chunk| {
                    body.extend_from_slice(&chunk);
                    Ok(body)
                })
                .and_then(|body| serde_json::from_slice(&body).map_err(|_| ()))
                .map(Json),
        ))
    }
}

impl<T: Serialize> Responder for Json<T> {
    type Error = serde_json::Error;

    fn respond(self) -> Result<Response, Self::Error> {
        let body = serde_json::to_string(&self.0)?;
        Ok(Response::new().with_header(ContentType::json()).with_body(
            body,
        ))
    }
}
