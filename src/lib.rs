extern crate finchers_core;
#[allow(unused_imports)]
#[macro_use]
extern crate finchers_derive;
extern crate finchers_json;
extern crate finchers_runtime;
extern crate finchers_urlencoded;

pub extern crate futures;
pub extern crate http;
pub extern crate mime;

#[doc(hidden)]
pub use finchers_derive::*;

pub use finchers_core::{endpoint, errors, request, response, service};
pub use finchers_core::errors::Error;

pub mod runtime {
    pub use finchers_runtime::{backend, test, Server};
    pub use finchers_runtime::backend::TcpBackend;
}

pub mod json {
    pub use finchers_json::{json_body, Error, Json, JsonBody, JsonResponder};
}

pub mod urlencoded {
    pub use finchers_urlencoded::{form_body, from_csv, queries, queries_opt, queries_req, Error, Form, FormBody,
                                  Queries, QueriesOptional, QueriesRequired};
}

pub mod prelude {
    pub use finchers_core::endpoint::Endpoint;
    pub use finchers_core::service::EndpointServiceExt;
    pub use finchers_runtime::test::EndpointTestExt;
}

#[macro_use]
mod macros;
