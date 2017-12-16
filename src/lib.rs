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
extern crate url;

pub mod endpoint;
pub mod request;
pub mod response;
pub mod server;
pub mod service;
pub mod task;
pub mod test;
pub mod util;

mod context;

pub use context::Context;

#[doc(inline)]
pub use endpoint::Endpoint;

#[doc(inline)]
pub use request::Request;

#[doc(inline)]
pub use response::{Responder, Response};

#[doc(inline)]
pub use server::ServerBuilder;

#[doc(inline)]
pub use task::{IntoTask, Task};
