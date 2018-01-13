//! Utility macros.

macro_rules! try_opt {
    ($e:expr) => {
        match $e {
            Some(e) => e,
            None => return None,
        }
    };
}

/// Convert the expression(s) into an `Endpoint`.
#[macro_export]
macro_rules! e {
    ($e:expr) => {
        $crate::IntoEndpoint::into_endpoint($e)
    };
    ($e:expr => <$a:ty, $b:ty>) => {
        $crate::IntoEndpoint::<$a, $b>::into_endpoint($e)
    };
}

/// Creates an `Endpoint` from multiple routes.
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
        let a = e!("foo" => <(), ()>);
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
