//! Components for checking the HTTP method

use endpoint::{Endpoint, EndpointContext, Input, IntoEndpoint};
use http::Method;

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchMethod<E: Endpoint> {
    method: Method,
    endpoint: E,
}

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Item = E::Item;
    type Result = E::Result;

    fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result> {
        if *input.method() == self.method {
            self.endpoint.apply(input, ctx)
        } else {
            None
        }
    }
}

#[allow(missing_docs)]
pub fn method<E: IntoEndpoint>(method: Method, endpoint: E) -> MatchMethod<E::Endpoint> {
    MatchMethod {
        method,
        endpoint: endpoint.into_endpoint(),
    }
}

macro_rules! define_method {
    ($(
        ($name:ident, $method:ident, $Endpoint:ident),
    )*) => {$(
        #[allow(missing_docs)]
        pub fn $name<E: IntoEndpoint>(endpoint: E) -> $Endpoint<E::Endpoint> {
            $Endpoint {
                endpoint: endpoint.into_endpoint(),
            }
        }

        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone)]
        pub struct $Endpoint<E> {
            endpoint: E,
        }

        impl<E: Endpoint> Endpoint for $Endpoint<E> {
            type Item = E::Item;
            type Result = E::Result;

            fn apply(&self, input: &Input, ctx: &mut EndpointContext) -> Option<Self::Result> {
                if *input.method() == Method::$method {
                    self.endpoint.apply(input, ctx)
                } else {
                    None
                }
            }
        }
    )*};
}

define_method! {
    (get, GET, MatchGet),
    (post, POST, MatchPost),
    (put, PUT, MatchPut),
    (delete, DELETE, MatchDelete),
    (head, HEAD, MatchHead),
    (patch, PATCH, MatchPatch),
    (trace, TRACE, MatchTrace),
    (connect, CONNECT, MatchConnect),
    (options, OPTIONS, MatchOptions),
}
