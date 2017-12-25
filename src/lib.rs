#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! Finchers is a combinator library for building HTTP services, based on
//! Hyper and Futures.

#[macro_use]
extern crate futures;
extern crate hyper;
extern crate net2;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

pub mod contrib;
pub mod endpoint;
pub mod request;
pub mod response;
pub mod service;
pub mod task;
pub mod test;


#[doc(inline)]
pub use endpoint::{Endpoint, EndpointContext, IntoEndpoint, NotFound};

#[doc(inline)]
pub use request::Request;

#[doc(inline)]
pub use response::Responder;

#[doc(inline)]
pub use task::{Task, TaskContext};
