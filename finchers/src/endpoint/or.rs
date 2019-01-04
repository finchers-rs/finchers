use {
    crate::{
        endpoint::{
            ActionContext, //
            Apply,
            ApplyContext,
            Endpoint,
            EndpointAction,
            IsEndpoint,
        },
        error::Error,
        output::IntoResponse,
    },
    either::Either::{self, *},
    futures::Poll,
    http::{Request, Response},
    std::mem,
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Or<E1, E2> {
    pub(super) e1: E1,
    pub(super) e2: E2,
}

impl<E1: IsEndpoint, E2: IsEndpoint> IsEndpoint for Or<E1, E2> {}

impl<E1, E2, Bd> Endpoint<Bd> for Or<E1, E2>
where
    E1: Endpoint<Bd>,
    E2: Endpoint<Bd>,
{
    type Output = (Wrapped<E1::Output, E2::Output>,);
    type Error = Error;
    type Action = OrAction<E1::Action, E2::Action>;

    fn apply(&self, ecx: &mut ApplyContext<'_, Bd>) -> Apply<Bd, Self> {
        let orig_cursor = ecx.cursor().clone();
        match self.e1.apply(ecx) {
            Ok(future1) => {
                let cursor1 = mem::replace(ecx.cursor(), orig_cursor);
                match self.e2.apply(ecx) {
                    Ok(future2) => {
                        // If both endpoints are matched, the one with the larger number of
                        // (consumed) path segments is choosen.
                        if cursor1.popped() >= ecx.cursor().popped() {
                            *ecx.cursor() = cursor1;
                            Ok(OrAction::left(future1))
                        } else {
                            Ok(OrAction::right(future2))
                        }
                    }
                    Err(..) => {
                        *ecx.cursor() = cursor1;
                        Ok(OrAction::left(future1))
                    }
                }
            }
            Err(..) => {
                let _ = mem::replace(ecx.cursor(), orig_cursor);
                match self.e2.apply(ecx) {
                    Ok(future) => Ok(OrAction::right(future)),
                    // FIXME: appropriate error handling
                    Err(..) => Err(http::StatusCode::NOT_FOUND.into()),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Wrapped<L, R>(Either<L, R>);

impl<L, R> IntoResponse for Wrapped<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    type Body = Either<L::Body, R::Body>;

    #[inline]
    fn into_response(self, request: &Request<()>) -> Response<Self::Body> {
        self.0.into_response(request)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct OrAction<L, R> {
    inner: Either<L, R>,
}

impl<L, R> OrAction<L, R> {
    fn left(l: L) -> Self {
        OrAction {
            inner: Either::Left(l),
        }
    }

    fn right(r: R) -> Self {
        OrAction {
            inner: Either::Right(r),
        }
    }
}

impl<L, R, Bd> EndpointAction<Bd> for OrAction<L, R>
where
    L: EndpointAction<Bd>,
    R: EndpointAction<Bd>,
{
    type Output = (Wrapped<L::Output, R::Output>,);
    type Error = Error;

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        match self.inner {
            Left(ref mut t) => t
                .poll_action(cx)
                .map(|t| t.map(|t| (Wrapped(Left(t)),)))
                .map_err(Into::into),
            Right(ref mut t) => t
                .poll_action(cx)
                .map(|t| t.map(|t| (Wrapped(Right(t)),)))
                .map_err(Into::into),
        }
    }
}

/// A helper macro for creating the instance of`Endpoint` from multiple routes.
///
/// # Example
///
/// ```ignore
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