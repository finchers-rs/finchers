#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! Finchers is a combinator library for building HTTP services, based on
//! Hyper and Futures.

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate futures;
extern crate hyper;
extern crate net2;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate url;

pub mod endpoint;
pub mod errors;
pub mod request;
pub mod response;
pub mod server;
pub mod test;
pub mod util;

mod context;
mod json;


pub use context::Context;

#[doc(inline)]
pub use endpoint::{Endpoint, NewEndpoint};

#[doc(inline)]
pub use request::Request;

#[doc(inline)]
pub use response::{Responder, Response};

#[doc(inline)]
pub use server::Server;

#[doc(inline)]
pub use json::Json;
