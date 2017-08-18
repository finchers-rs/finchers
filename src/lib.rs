#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;

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
            EndpointError, EndpointErrorKind, ResultExt, EndpointResult;
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
