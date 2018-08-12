/// A helper macro for creating the instance of`Endpoint` from multiple routes.
///
/// # Example
///
/// ```ignore
/// #![feature(async_await)]
/// #![feature(rust_2018_preview)]
///
/// # use finchers_core::{route, routes};
/// # use finchers_core::endpoint::EndpointExt;
/// # use finchers_core::endpoints::body::body;
/// #
/// let get_post = route!(@get / i32)
///     .and_then(async move |id| {
///         Ok((format!("get_post: {}", id),))
///     });
///
/// let add_post = route!(@post /).and(body::<String>())
///     .and_then(async move |data| {
///         Ok((format!("add_post: {}", data),))
///     });
///
/// // ...
///
/// let endpoint = route!(/ "posts").and(routes![
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
        routes!(@inner $e1, routes!(@inner $e2, $($t),*))
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

/// A helper macro for creating an endpoint from the specified segments.
///
/// # Example
///
/// The following macro call
///
/// ```ignore
/// route!(@get / "api" / "v1" / "posts" / i32);
/// ```
///
/// will be roughly expanded to:
///
/// ```ignore
/// method::get(
///     path("api")
///         .and(path("v1"))
///         .and(path("posts"))
///         .and(param::<i32>())
/// )
/// ```
#[macro_export]
macro_rules! route {
    (@$method:ident / $($t:tt)*) => ( route!(@@start $method / $($t)*) );
    (/ $($t:tt)*) => ( route!(@@start get / $($t)*) );

    (@@start $method:ident / $head:tt $(/ $tail:tt)*) => {
        $crate::endpoints::method::$method({
            let __p = route!(@@segment $head);
            $(
                let __p = $crate::endpoint::EndpointExt::and(__p, route!(@@segment $tail));
            )*
            __p
        })
    };
    (@@start $method:ident / $head:tt $(/ $tail:tt)* /) => {
        route!(@@start $method $head $(/ $tail)*)
    };
    (@@start $method:ident /) => {
        $crate::endpoints::method::$method({
            $crate::endpoint::ok(())
        })
    };
    (@@segment $t:ty) => ( $crate::endpoints::path::param::<$t>() );
    (@@segment $s:expr) => ( $crate::endpoints::path::path($s) );
}

#[cfg(test)]
mod tests {
    use endpoints::path::path;

    #[test]
    #[allow(unused_variables)]
    fn compile_test_route() {
        let e1 = route!(@get /);
        let e2 = route!(@get / "foo" / String / "bar");
        let e3 = route!(@get / i32);
    }

    #[test]
    #[allow(unused_variables)]
    fn compile_test_routes() {
        let e1 = path("foo");
        let e2 = routes!(e1, path("bar"), path("baz"));
        let e3 = routes!(path("foobar"), e2);
        let e4 = routes!(path("foobar"), e3,);
    }
}
