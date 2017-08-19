#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;
extern crate serde;
extern crate serde_json;

pub mod combinator;
pub mod context;
pub mod endpoint;
pub mod request;
pub mod response;
pub mod server;
pub mod test;

pub mod errors {
    use context::Context;
    use request::Body;

    error_chain! {
        types {
            EndpointError, EndpointErrorKind, EndpointResultExt, _EndpointResult;
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

    pub type EndpointResult<'r, F> = Result<(Context<'r>, Option<Body>, F), (EndpointError, Option<Body>)>;
}

#[doc(inline)]
pub use context::Context;
#[doc(inline)]
pub use endpoint::{Endpoint, NewEndpoint};
#[doc(inline)]
pub use response::{Response, Responder};
#[doc(inline)]
pub use server::EndpointService;
