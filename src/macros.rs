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
    () => { $crate::routes_impl!(@error); };
    ($h:expr) => {  $crate::routes_impl!(@error); };
    ($h:expr,) => {  $crate::routes_impl!(@error); };
    ($e1:expr, $e2:expr) => { $crate::routes_impl!($e1, $e2); };
    ($e1:expr, $e2:expr,) => { $crate::routes_impl!($e1, $e2); };
    ($e1:expr, $e2:expr, $($t:expr),*) => { $crate::routes_impl!($e1, $e2, $($t),*); };
    ($e1:expr, $e2:expr, $($t:expr,)+) => { $crate::routes_impl!($e1, $e2, $($t),+); };
}

#[doc(hidden)]
#[macro_export]
macro_rules! routes_impl {
    ($e1:expr, $e2:expr, $($t:expr),*) => {{
        $crate::routes_impl!($e1, $crate::routes_impl!($e2, $($t),*))
    }};

    ($e1:expr, $e2:expr) => {{
        $crate::endpoint::EndpointExt::or(
            $crate::endpoint::EndpointExt::map(
                $crate::endpoint::IntoEndpoint::into_endpoint($e1),
                $crate::generic::map_left(),
            ),
            $crate::endpoint::EndpointExt::map(
                $crate::endpoint::IntoEndpoint::into_endpoint($e2),
                $crate::generic::map_right(),
            )
        )
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
    // with method
    (@$method:ident $($t:tt)*) => (
        $crate::endpoints::method::$method(
            $crate::route_impl!(@start $($t)*)
        )
    );

    // without method
    (/ $($t:tt)*) => ( $crate::route_impl!(@start / $($t)*) );
    () => ( $crate::route_impl!(@start) );
}

#[doc(hidden)]
#[macro_export]
macro_rules! route_impl {
    (@start / $head:tt $(/ $tail:tt)*) => {{
        let __p = $crate::route_impl!(@segment $head);
        $(
            let __p = $crate::endpoint::EndpointExt::and(__p, $crate::route_impl!(@segment $tail));
        )*
        __p
    }};
    (@start / $head:tt $(/ $tail:tt)* /) => {
        $crate::route_impl!(@start / $head $(/ $tail)*)
            .and($crate::endpoints::path::end())
    };
    (@start /) => ( $crate::endpoints::path::end() );
    (@start) => ( $crate::endpoint::unit() );

    (@segment $t:ty) => ( $crate::endpoints::path::param::<$t>() );
    (@segment $s:expr) => ( $crate::endpoints::path::path($s) );
}

#[cfg(test)]
mod tests {
    use crate::endpoint::EndpointExt;
    use crate::endpoints::path::path;

    #[test]
    fn compile_test_route() {
        let _ = route!().output::<()>();
        let _ = route!(/).output::<()>();
        let _ = route!(/"foo"/i32).output::<(i32,)>();

        let _ = route!(@get /).output::<()>();
        let _ = route!(@get / "foo" / String / "bar").output::<(String,)>();
        let _ = route!(@get / "foo" / String / i32 / "bar" /).output::<(String, i32)>();
        let _ = route!(@get / i32).output::<(i32,)>();
        let _ = route!(@get / i32 / ).output::<(i32,)>();
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
