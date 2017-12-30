//! A combinator library for building asynchronous HTTP services.

#![doc(html_root_url = "https://docs.rs/finchers/0.10.1")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(warnings)]

extern crate cookie;
#[macro_use]
extern crate futures;
extern crate hyper;
extern crate net2;
extern crate tokio_core;

pub mod contrib;
pub mod endpoint;
pub mod http;
pub mod responder;
pub mod service;
pub mod task;
pub mod test;

#[doc(inline)]
pub use endpoint::{Endpoint, IntoEndpoint};

#[doc(inline)]
pub use responder::{ErrorResponder, IntoResponder, Responder};

#[doc(inline)]
pub use task::Task;
