//! Utility macros.

macro_rules! try_opt {
    ($e:expr) => {
        match $e {
            Some(e) => e,
            None => return None,
        }
    };
}

/// A helper macro for creating the instance of`Endpoint`.
#[macro_export]
macro_rules! endpoint {
    ($e:expr) => {
        $crate::IntoEndpoint::into_endpoint($e)
    };
    ($e:expr => <$a:ty, $b:ty>) => {
        $crate::IntoEndpoint::<$a, $b>::into_endpoint($e)
    };
    ($h:expr, $($t:expr),*) => {
        $crate::IntoEndpoint::into_endpoint($h)
            $( .or($t) )*
    };
    ($h:expr, $($t:expr,)+) => {
        endpoint!($h, $($t),*)
    };
}

#[cfg(test)]
mod tests {
    use Endpoint;

    #[test]
    #[allow(unused_variables)]
    fn compile_test_e() {
        let a = endpoint!("foo").with_type::<(), ()>();
        let b = endpoint!("foo" => <(), ()>);
    }

    #[test]
    #[allow(unused_variables)]
    fn compile_test_choice() {
        let e1 = endpoint!("foo");
        let e2 = endpoint!(e1, "bar", "baz");
        let e3 = endpoint!("foobar", e2,);
        let e4 = e3.with_type::<_, ()>();
    }
}
