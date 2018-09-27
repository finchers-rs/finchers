use either::Either;
use either::Either::*;
use http::Response;

use endpoint::{Context, Endpoint, EndpointResult};
use error::Error;
use output::{Output, OutputContext};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<'a, E1, E2> Endpoint<'a> for Or<E1, E2>
where
    E1: Endpoint<'a>,
    E2: Endpoint<'a>,
{
    type Output = (Wrapped<E1::Output, E2::Output>,);
    type Future = OrFuture<E1::Future, E2::Future>;

    fn apply(&'a self, ecx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        match {
            let mut ecx = ecx.clone_reborrowed();
            self.e1
                .apply(&mut ecx)
                .map(|future| (future, ecx.current_cursor()))
        } {
            Ok((future1, cursor1)) => {
                match {
                    let mut ecx = ecx.clone_reborrowed();
                    self.e2
                        .apply(&mut ecx)
                        .map(|future| (future, ecx.current_cursor()))
                } {
                    // If both endpoints are matched, the one with the larger number of
                    // (consumed) path segments is choosen.
                    Ok((_, ref cursor2)) if cursor1.popped >= cursor2.popped => {
                        ecx.reset_cursor(cursor1);
                        Ok(OrFuture::left(future1))
                    }
                    Ok((future2, cursor2)) => {
                        ecx.reset_cursor(cursor2);
                        Ok(OrFuture::right(future2))
                    }
                    Err(..) => {
                        ecx.reset_cursor(cursor1);
                        Ok(OrFuture::left(future1))
                    }
                }
            }
            Err(err1) => match self.e2.apply(ecx) {
                Ok(future) => Ok(OrFuture::right(future)),
                Err(err2) => Err(err1.merge(err2)),
            },
        }
    }
}

#[derive(Debug)]
pub struct Wrapped<L, R>(Either<L, R>);

impl<L: Output, R: Output> Output for Wrapped<L, R> {
    type Body = Either<L::Body, R::Body>;
    type Error = Error;

    #[inline(always)]
    fn respond(self, cx: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        self.0.respond(cx)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrFuture<L, R> {
    inner: Either<L, R>,
}

impl<L, R> OrFuture<L, R> {
    fn left(l: L) -> Self {
        OrFuture {
            inner: Either::Left(l),
        }
    }

    fn right(r: R) -> Self {
        OrFuture {
            inner: Either::Right(r),
        }
    }
}

impl<L, R> ::futures::Future for OrFuture<L, R>
where
    L: ::futures::Future<Error = Error>,
    R: ::futures::Future<Error = Error>,
{
    type Item = (Wrapped<L::Item, R::Item>,);
    type Error = Error;

    #[inline(always)]
    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        match self.inner {
            Left(ref mut t) => t.poll().map(|t| t.map(|t| (Wrapped(Left(t)),))),
            Right(ref mut t) => t.poll().map(|t| t.map(|t| (Wrapped(Right(t)),))),
        }
    }
}

/// A helper macro for creating the instance of`Endpoint` from multiple routes.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate finchers;
///
/// use finchers::prelude::*;
///
/// # fn main() {
/// let get_post = path!(@get / i32 /)
///     .map(|id| format!("get_post: {}", id));
///
/// let add_post = path!(@post /)
///     .and(endpoints::body::text())
///     .map(|data: String| format!("add_post: {}", data));
///
/// // ...
///
/// let endpoint = path!(/ "posts")
///     .and(routes![
///         get_post,
///         add_post,
///         // ...
///     ]);
/// # drop(endpoint);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! routes {
    () => { routes_impl!(@error); };
    ($h:expr) => { routes_impl!(@error); };
    ($h:expr,) => { routes_impl!(@error); };
    ($e1:expr, $e2:expr) => { routes_impl!($e1, $e2); };
    ($e1:expr, $e2:expr,) => { routes_impl!($e1, $e2); };
    ($e1:expr, $e2:expr, $($t:expr),*) => { routes_impl!($e1, $e2, $($t),*); };
    ($e1:expr, $e2:expr, $($t:expr,)+) => { routes_impl!($e1, $e2, $($t),+); };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! routes_impl {
    ($e1:expr, $e2:expr, $($t:expr),*) => {{
        routes_impl!($e1, routes_impl!($e2, $($t),*))
    }};

    ($e1:expr, $e2:expr) => {{
        $crate::endpoint::IntoEndpointExt::or(
            $crate::endpoint::IntoEndpoint::into_endpoint($e1),
            $crate::endpoint::IntoEndpoint::into_endpoint($e2),
        )
    }};

    (@error) => { compile_error!("The `routes!()` macro requires at least two elements."); };
}
