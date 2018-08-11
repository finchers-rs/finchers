/// A helper macro for creating the instance of`Endpoint` from multiple routes.
///
/// # Example
///
/// ```
/// #![feature(async_await)]
/// #![feature(rust_2018_preview)]
///
/// # use finchers_core::routes;
/// # use finchers_core::endpoint::EndpointExt;
/// # use finchers_core::endpoints::body::body;
/// # use finchers_core::endpoints::path::{path, param};
/// # use finchers_core::endpoints::method;
/// #
/// let get_post = method::get(param())
///     .and_then(async move |id: u32| {
///         Ok((format!("get_post: {}", id),))
///     });
///
/// let add_post = method::post(body())
///     .and_then(async move |data: String| {
///         Ok((format!("add_post: {}", data),))
///     });
///
/// // ...
///
/// let endpoint = path("posts").and(routes![
///     get_post,
///     add_post,
///     // ...
/// ]);
/// ```
#[macro_export]
macro_rules! routes {
    () => { routes!(@error); };
    ($h:expr) => {  routes!(@error); };
    ($h:expr,) => {  routes!(@error); };
    ($e1:expr, $e2:expr) => { routes!(@inner $e1, $e2); };
    ($e1:expr, $e2:expr,) => { routes!(@inner $e1, $e2); };
    ($e1:expr, $e2:expr, $($t:expr),*) => { routes!(@inner $e1, $e2, $($t),*); };
    ($e1:expr, $e2:expr, $($t:expr,)+) => { routes!(@inner $e1, $e2, $($t),+); };

    (@inner $e1:expr, $e2:expr, $($t:expr),*) => {{
        #[allow(unused_imports)]
        use $crate::endpoint::{IntoEndpoint, EndpointExt};
        #[allow(unused_imports)]
        use $crate::generic::{map_left, map_right};

        routes!{ @inner
            IntoEndpoint::into_endpoint($e1).map_ok(map_left())
                .or(IntoEndpoint::into_endpoint($e2).map_ok(map_right())),
            $($t),*
        }
    }};

    (@inner $e1:expr, $e2:expr) => {{
        use $crate::endpoint::IntoEndpoint;
        use $crate::endpoint::EndpointExt;
        use $crate::generic::{map_left, map_right};

        IntoEndpoint::into_endpoint($e1).map_ok(map_left())
            .or(IntoEndpoint::into_endpoint($e2).map_ok(map_right()))
    }};

    (@error) => { compile_error!("The `routes!()` macro requires at least two elements."); };
}

#[cfg(test)]
mod tests {
    use crate::endpoints::path::path;

    #[test]
    #[allow(unused_variables)]
    fn compile_test_routes() {
        let e1 = path("foo");
        let e2 = routes!(e1, path("bar"), path("baz"));
        let e3 = routes!(path("foobar"), e2);
        let e4 = routes!(path("foobar"), e3,);
    }
}

macro_rules! try_poll {
    ($e:expr) => {{
        use std::task::Poll;
        match $e {
            Poll::Ready(Ok(x)) => x,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(Into::into(e))),
            Poll::Pending => return Poll::Pending,
        }
    }};
}
