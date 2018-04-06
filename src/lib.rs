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

pub use finchers_core::error::Error;
pub use finchers_core::{error, input, response};

pub mod endpoint {
    pub use finchers_endpoint::*;

    /// The "prelude" for building endpoints
    pub mod prelude {
        pub use finchers_endpoint::body::{body, body_stream};
        pub use finchers_endpoint::header::{header, header_opt, header_req};
        pub use finchers_endpoint::method::{delete, get, head, patch, post, put};
        pub use finchers_endpoint::path::{match_, path, paths};
        pub use finchers_endpoint::{endpoint, Endpoint, IntoEndpoint};
    }
}

pub mod runtime {
    pub use finchers_runtime::*;
}

pub mod json {
    pub use finchers_json::{json_body, Error, Json, JsonBody, JsonResponder};
}

pub mod urlencoded {
    pub use finchers_urlencoded::{form_body, from_csv, queries, queries_opt, queries_req, Error, Form, FormBody,
                                  Queries, QueriesOptional, QueriesRequired};
}

pub mod prelude {
    pub use finchers_endpoint::Endpoint;
    pub use finchers_runtime::EndpointServiceExt;
}

#[macro_use]
mod macros;
