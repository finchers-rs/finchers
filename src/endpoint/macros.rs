//! Utility macros.

#[macro_export]
macro_rules! e {
    ($e:expr) => {
        $crate::IntoEndpoint::into_endpoint($e)
    };
    ($h:expr, $($t:expr),*) => {
        $crate::IntoEndpoint::into_endpoint(($h, $($t),*))
    };
    ($h:expr, $($t:expr,)+) => {
        e!($h, $($t),+)
    };
}

#[macro_export]
macro_rules! choice {
    ($e:expr) => {
        $crate::IntoEndpoint::into_endpoint($e)
    };
    ($h:expr, $($t:expr),*) => {
        $crate::IntoEndpoint::into_endpoint($h)
            $( .or($t) )*
    };
    ($h:expr, $($t:expr,)+) => {
        choice!($h, $($t),*)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[allow(unused_variables)]
    fn compile_test_e() {
        use Endpoint;
        let e = e!("foo").map_err(|e: ()| ());
        let e = e!("foo", "bar", "baz").map_err(|e: ()| ());
        let e = e!("foo", "bar", "baz",).map_err(|e: ()| ());
    }

    #[test]
    #[allow(unused_variables)]
    fn compile_test_choice() {
        use Endpoint;
        let foo = choice!("foo").map_err(|e: ()| ());
        let e = choice!(foo, "bar", "baz");
        let e = choice!("foobar", e,);
    }
}