//! Definitions and reexports of incoming HTTP requests

mod body;
mod request;


#[doc(inline)]
pub use self::body::{Body, FromBody, IntoVec};

#[doc(inline)]
pub use self::request::Request;


use hyper;

/// reconstruct the raw incoming HTTP request, and return a pair of `Request` and `Body`
pub fn reconstruct(req: hyper::Request) -> (Request, Body) {
    let (method, uri, _version, headers, body) = req.deconstruct();
    let req = Request {
        method,
        uri,
        headers,
    };
    (req, body.into())
}
