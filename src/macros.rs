//! Utility macros.

macro_rules! try_opt {
    ($e:expr) => {
        match $e {
            Some(e) => e,
            None => return None,
        }
    };
}

/// A helper macro for creating the instance of`Endpoint` from multiple routes.
///
/// # Example
///
/// A macro call
///
/// ```ignore
/// choice!(e1, e2, e3)
/// ```
///
/// will be expanded to
///
/// ```ignore
/// endpoint(e1).from_err()
///     .or(endpoint(e2).from_err())
///     .or(endpoint(e3).from_err())
/// ```
#[macro_export]
macro_rules! choice {
    ($h:expr, $($t:expr),*) => {{
        use $crate::endpoint::{endpoint, Endpoint};
        endpoint($h).from_err()
            $( .or(endpoint($t).from_err()) )*
    }};
    ($h:expr, $($t:expr,)+) => {
        choice!($h, $($t),*)
    };
}

#[cfg(test)]
mod tests {
    use endpoint::{endpoint, Endpoint};

    #[test]
    #[allow(unused_variables)]
    fn compile_test_choice() {
        let e1 = endpoint("foo");
        let e2 = choice!(e1, "bar", "baz");
        let e3 = choice!("foobar", e2,);
        let e4 = e3.assert_types::<_, ()>();
    }
}
