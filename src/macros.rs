#[doc(hidden)]
#[deprecated(since = "0.12.0-alpha.4", note = "use `path!()` instead.")]
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
