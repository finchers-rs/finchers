//! Components for checking the HTTP method.

use http::Method;

use crate::endpoint::{Context, Endpoint, EndpointErrorKind, EndpointResult, IntoEndpoint};

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchMethod<E> {
    method: Method,
    endpoint: E,
}

impl<'a, E: Endpoint<'a>> Endpoint<'a> for MatchMethod<E> {
    type Output = E::Output;
    type Future = E::Future;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        if *ecx.input().method() == self.method {
            self.endpoint.apply(ecx)
        } else {
            Err(EndpointErrorKind::MethodNotAllowed(vec![
                self.method.clone(),
            ]))
        }
    }
}

/// Create an endpoint which will accept the request only if the request method is equal to the expected one.
pub fn method<E>(method: Method, endpoint: E) -> MatchMethod<E>
where
    for<'e> E: Endpoint<'e>,
{
    MatchMethod { method, endpoint }
}

macro_rules! define_method {
    ($(
        $(#[$doc:meta])*
        ($name:ident, $method:ident, $Endpoint:ident),
    )*) => {$(

        $(#[$doc])*
        pub fn $name<E>(endpoint: E) -> $Endpoint<E>
        where
            for<'e> E: Endpoint<'e>,
        {
            $Endpoint {
                endpoint: endpoint.into_endpoint(),
            }
        }

        #[allow(missing_docs)]
        #[derive(Debug, Copy, Clone)]
        pub struct $Endpoint<E> {
            endpoint: E,
        }

        impl<'e, E: Endpoint<'e>> Endpoint<'e> for $Endpoint<E> {
            type Output = E::Output;
            type Future = E::Future;

            fn apply(&'e self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
                if *ecx.input().method() == Method::$method {
                    self.endpoint.apply(ecx)
                } else {
                    Err(EndpointErrorKind::MethodNotAllowed(vec![Method::$method]))
                }
            }
        }
    )*};
}

define_method! {
    /// Create an endpoint which will accept the request only if the request method is equal to
    /// `GET`.
    (get, GET, MatchGet),

    /// Create an endpoint which will accept the request only if the request method is equal to
    /// `POST`.
    (post, POST, MatchPost),

    /// Create an endpoint which will accept the request only if the request method is equal to
    /// `PUT`.
    (put, PUT, MatchPut),

    /// Create an endpoint which will accept the request only if the request method is equal to
    /// `DELETE`.
    (delete, DELETE, MatchDelete),

    /// Create an endpoint which will accept the request only if the request method is equal to
    /// `HEAD`.
    (head, HEAD, MatchHead),

    /// Create an endpoint which will accept the request only if the request method is equal to
    /// `PATCH`.
    (patch, PATCH, MatchPatch),
}
