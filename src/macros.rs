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
/// endpoint(e1)
///     .or(endpoint(e2))
///     .or(endpoint(e3))
/// ```
#[macro_export]
macro_rules! choice {
    ($h:expr, $($t:expr),*) => {{
        use $crate::endpoint::{endpoint, Endpoint};
        endpoint($h)
            $( .or(endpoint($t)) )*
    }};
    ($h:expr, $($t:expr,)+) => {
        choice!($h, $($t),*)
    };
}

#[cfg(test)]
mod tests {
    use endpoint::endpoint;

    #[test]
    #[allow(unused_variables)]
    fn compile_test_choice() {
        let e1 = endpoint("foo");
        let e2 = choice!(e1, "bar", "baz");
        let e3 = choice!("foobar", e2,);
    }
}
