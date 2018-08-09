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
        use $crate::endpoint::IntoEndpoint;
        use $crate::endpoint::ext::EndpointExt;
        IntoEndpoint::into_endpoint($h)
            $( .or(IntoEndpoint::into_endpoint($t)) )*
    }};
    ($h:expr, $($t:expr,)+) => {
        choice!($h, $($t),*)
    };
}

#[cfg(test)]
mod tests {
    use crate::http::path::path;

    #[test]
    #[allow(unused_variables)]
    fn compile_test_choice() {
        let e1 = path("foo");
        let e2 = choice!(e1, path("bar"), path("baz"));
        let e3 = choice!(path("foobar"), e2,);
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
