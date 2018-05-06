//! Components for checking the HTTP method.

use finchers_core::endpoint::{Context, Endpoint, IntoEndpoint};
use http::Method;

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct MatchMethod<E: Endpoint> {
    method: Method,
    endpoint: E,
}

impl<E: Endpoint> Endpoint for MatchMethod<E> {
    type Output = E::Output;
    type Task = E::Task;

    fn apply(&self, cx: &mut Context) -> Option<Self::Task> {
        if *cx.input().request().method() == self.method {
            self.endpoint.apply(cx)
        } else {
            None
        }
    }
}

/// Create an endpoint which will accept the request only if the request method is equal to the expected one.
pub fn method<E>(method: Method, endpoint: E) -> MatchMethod<E::Endpoint>
where
    E: IntoEndpoint,
{
    MatchMethod {
        method,
        endpoint: endpoint.into_endpoint(),
    }
}

macro_rules! define_method {
    ($(
        $(#[$doc:meta])*
        ($name:ident, $method:ident, $Endpoint:ident),
    )*) => {$(

        $(#[$doc])*
        pub fn $name<E>(endpoint: E) -> $Endpoint<E::Endpoint>
        where
            E: IntoEndpoint,
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

        impl<E: Endpoint> Endpoint for $Endpoint<E> {
            type Output = E::Output;
            type Task = E::Task;

            fn apply(&self,cx: &mut Context) -> Option<Self::Task> {
                if *cx.input().request().method() == Method::$method {
                    self.endpoint.apply(cx)
                } else {
                    None
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
