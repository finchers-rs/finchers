#![deny(missing_docs)]

//! Finchers is a combinator library for building HTTP services, based on
//! Hyper and Futures.

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;
extern crate serde;
extern crate serde_json;

pub mod combinator;
pub mod endpoint;
pub mod errors;
pub mod request;
pub mod response;
pub mod server;
pub mod test;

mod context;
mod json;

pub use context::Context;

#[doc(inline)]
pub use endpoint::Endpoint;

#[doc(inline)]
pub use request::{Request, Body};

#[doc(inline)]
pub use response::{Response, Responder};

#[doc(inline)]
pub use server::EndpointService;

#[doc(inline)]
pub use json::Json;
