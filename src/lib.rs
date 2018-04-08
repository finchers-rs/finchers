extern crate finchers_core;
#[allow(unused_imports)]
#[macro_use]
extern crate finchers_derive;
extern crate finchers_endpoint;
extern crate finchers_json;
extern crate finchers_runtime;
extern crate finchers_urlencoded;

pub extern crate futures;
pub extern crate http;
pub extern crate mime;

#[doc(hidden)]
pub use finchers_derive::*;

pub use finchers_core::{error, output};

pub mod endpoint {
    pub use finchers_endpoint::{all, body, endpoint, header, method, ok, path, skip_all, Endpoint, EndpointExt,
                                FromBody, FromHeader, FromSegment, FromSegments, IntoEndpoint};

    /// The "prelude" for building endpoints
    pub mod prelude {
        pub use finchers_endpoint::body::{body, body_stream};
        pub use finchers_endpoint::header::{header, header_opt, header_req};
        pub use finchers_endpoint::method::{delete, get, head, patch, post, put};
        pub use finchers_endpoint::path::{match_, path, paths};
        pub use finchers_endpoint::{endpoint, Endpoint, IntoEndpoint};
    }
}

pub mod input {
    pub use finchers_core::input::{Body, BodyStream, Error, ErrorKind, Input};
}

pub mod runtime {
    pub use finchers_runtime::{EndpointServiceExt, FinchersService, FinchersServiceFuture, HttpService, Server};
}

pub mod json {
    pub use finchers_json::{json_body, Error, Json, JsonBody, JsonOutput};
}

pub mod urlencoded {
    pub use finchers_urlencoded::{form_body, from_csv, queries, queries_opt, queries_req, Error, Form, FormBody,
                                  Queries, QueriesOptional, QueriesRequired};
}

pub mod prelude {
    pub use finchers_endpoint::EndpointExt;
    pub use finchers_runtime::EndpointServiceExt;
}

pub use finchers_core::{Error, Input, Output};
pub use finchers_endpoint::{Endpoint, EndpointExt};

#[macro_use]
mod macros;
