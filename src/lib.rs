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
pub mod context;
pub mod either;
pub mod endpoint;
pub mod request;
pub mod response;
pub mod server;
pub mod test;

pub mod errors {
    error_chain! {
        types {
            EndpointError, EndpointErrorKind, EndpointResultExt, EndpointResult;
        }
        errors {
            NoRoute {
                description("no route")
                display("no route")
            }
            InvalidMethod {
                description("invalid method")
                display("invalid method")
            }
            RemainingPath {
                description("remaining path")
                display("remaining path")
            }
        }
    }
}

#[doc(inline)]
pub use context::Context;
#[doc(inline)]
pub use endpoint::{Endpoint, NewEndpoint};
#[doc(inline)]
pub use errors::*;
#[doc(inline)]
pub use response::{Response, Responder};
#[doc(inline)]
pub use server::EndpointService;
