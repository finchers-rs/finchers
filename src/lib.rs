#![warn(missing_docs)]
//#![deny(warnings)]
#![deny(missing_debug_implementations)]

//! An HTTP routing library for Rust.
//!
//! This library is aimed to construct the Web application
//! by combining the multiple "basic" components.
//! The application are built "declaratively", and there is
//! no need to explicitly describe the details of raw
//! request processing (like parsing the path segments).
//!
//! There are three main layer in the constructed Web application:
//!
//! * `Endpoint` - The abstruction of the Web application for
//!   routing and creating the instance of `Task`
//! * `Task` - The body of processing the incoming request
//! * `Responder` - Coverting the result/error value returned
//!   from `Task` into a "raw" HTTP response
//!
//! The trait `Task` is very similar to `Future`, but it can
//! take an argument to access the context during request handling.
//!
//! # Example
//! ```
//! # #[macro_use]
//! # extern crate error_chain;
//! extern crate finchers;
//!
//! use finchers::prelude::*;
//! use finchers::endpoint::{body, param};
//! use finchers::endpoint::method::{get, post};
//!
//! # fn main() {
//! // GET /hello/:name
//! let hello = get(("hello", param()))
//!     .and_then(|(_, name): (_, String)| {
//!         Ok(format!("Hello, {}", name))
//!     });
//!
//! // POST /add [text/plain; charset=utf-8]
//! let with_body = post(("add", body()))
//!     .and_then(|(_, body): (_, String)| {
//!         Ok(format!("Received: {}", body))
//!     });
//!
//! // The root endpoint
//! let endpoint = hello.or(with_body)
//!     .with_type::<_, MyError>();
//!
//! error_chain! {
//!     types { MyError, MyErrorKind, MyResult; }
//!     foreign_links {
//!         Routing(EndpointError);
//!         BodyReceiving(finchers::request::BodyError);
//!         BodyParsing(std::string::FromUtf8Error);
//!     }
//! }
//!
//! impl Responder for MyError {
//!     // ...
//! #    fn respond_to(&mut self, _: &mut ResponderContext) -> Response {
//! #        unimplemented!()
//! #    }
//! }
//!
//! // Uncomment to serve the created endpoint
//! // Server::default().serve(endpoint)
//!
//! # }
//! ```


#[macro_use]
extern crate futures;
extern crate net2;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate url;
#[doc(hidden)]
pub extern crate hyper;

pub mod endpoint;
pub mod request;
pub mod response;
pub mod server;
pub mod task;
pub mod test;

pub use endpoint::Endpoint;
pub use request::{Body, Request};
pub use response::{Responder, Response};
pub use server::Server;
pub use task::Task;

/// A "prelude" re-exports
pub mod prelude {
    pub use endpoint::{Endpoint, EndpointContext, EndpointError};
    pub use endpoint::method;
    pub use task::{Async, Poll, Task, TaskContext};
    pub use response::{Responder, ResponderContext, Response, ResponseBuilder};
    pub use server::Server;
}
