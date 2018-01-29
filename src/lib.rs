//! A combinator library for building asynchronous HTTP services.

#![doc(html_root_url = "https://docs.rs/finchers/0.10.1")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(warnings)]

#[macro_use]
extern crate futures;
extern crate http as http_crate;
extern crate hyper;
#[cfg(feature = "tls")]
extern crate native_tls;
extern crate net2;
extern crate tokio_core;
extern crate tokio_io;
#[cfg(feature = "tls")]
extern crate tokio_tls;

#[macro_use]
mod macros;

pub mod application;
pub mod endpoint;
pub mod errors;
pub mod http;
pub mod handler;
pub mod responder;
pub mod service;
pub mod test;

#[doc(inline)]
pub use application::Application;

#[doc(inline)]
pub use endpoint::{Endpoint, EndpointResult, IntoEndpoint};

#[doc(inline)]
pub use errors::{Error, HttpError};

#[doc(inline)]
pub use handler::Handler;

#[doc(inline)]
pub use responder::Responder;

#[doc(inline)]
pub use service::EndpointServiceExt;
