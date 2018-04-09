extern crate finchers_core;
#[allow(unused_imports)]
#[macro_use]
extern crate finchers_derive;
extern crate finchers_endpoint;
extern crate finchers_http;
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
    pub use finchers_core::endpoint::{Endpoint, IntoEndpoint};
    pub use finchers_endpoint::{all, ok, skip_all, EndpointExt};

    pub use finchers_http::{body, header, method, path, FromBody, FromHeader, FromSegment, FromSegments};

    /// The "prelude" for building endpoints
    pub mod prelude {
        pub use finchers_core::endpoint::{Endpoint, IntoEndpoint};
        pub use finchers_http::body::{body, body_stream};
        pub use finchers_http::header::header;
        pub use finchers_http::method::{delete, get, head, patch, post, put};
        pub use finchers_http::path::{param, params, path};
    }
}

pub mod input {
    pub use finchers_core::input::{Body, BodyStream, Error, ErrorKind, Input};
}

pub mod runtime {
    pub use finchers_runtime::{EndpointServiceExt, FinchersService, FinchersServiceFuture, HttpService, Server};
}

pub mod json {
    pub use finchers_json::{Error, Json};
}

pub mod urlencoded {
    pub use finchers_urlencoded::{from_csv, queries, queries_opt, queries_req, Error, Form, Queries, QueriesOptional,
                                  QueriesRequired};
}

pub mod prelude {
    pub use finchers_endpoint::EndpointExt;
    pub use finchers_runtime::EndpointServiceExt;
}

pub use finchers_core::endpoint::Endpoint;
pub use finchers_core::{Input, Output};
pub use finchers_endpoint::EndpointExt;

#[macro_use]
mod macros;
