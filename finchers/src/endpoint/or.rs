use {
    crate::{
        endpoint::{
            ActionContext, //
            ApplyContext,
            Endpoint,
            EndpointAction,
            IsEndpoint,
            Preflight,
        },
        error::Error,
        output::IntoResponse,
    },
    either::Either,
    futures::Poll,
    http::{Request, Response},
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

    fn action(&self) -> Self::Action {
        OrAction {
            state: State::Init(self.e1.action(), self.e2.action()),
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

#[allow(missing_debug_implementations)]
enum State<L, R> {
    Init(L, R),
    Left(L),
    Right(R),
    Done,
}

#[allow(missing_debug_implementations)]
pub struct OrAction<L, R> {
    state: State<L, R>,
}

impl<L, R, Bd> EndpointAction<Bd> for OrAction<L, R>
where
    L: EndpointAction<Bd>,
    R: EndpointAction<Bd>,
{
    type Output = (Wrapped<L::Output, R::Output>,);
    type Error = Error;

    fn preflight(
        &mut self,
        cx: &mut ApplyContext<'_>,
    ) -> Result<Preflight<Self::Output>, Self::Error> {
        self.state = match std::mem::replace(&mut self.state, State::Done) {
            State::Init(mut left, mut right) => {
                let orig_cx = cx.clone();
                let left_output = left.preflight(cx);
                let cx1 = std::mem::replace(cx, orig_cx);
                let right_output = right.preflight(cx);

                match (left_output, right_output) {
                    (Ok(l), Ok(r)) => {
                        // If both endpoints are matched, the one with the larger number of
                        // (consumed) path segments is choosen.
                        if cx1.num_popped_segments() >= cx.num_popped_segments() {
                            *cx = cx1;
                            if let Preflight::Completed(output) = l {
                                return Ok(Preflight::Completed((Wrapped(Either::Left(output)),)));
                            } else {
                                State::Left(left)
                            }
                        } else if let Preflight::Completed(output) = r {
                            return Ok(Preflight::Completed((Wrapped(Either::Right(output)),)));
                        } else {
                            State::Right(right)
                        }
                    }

                    (Ok(l), Err(..)) => {
                        *cx = cx1;
                        if let Preflight::Completed(output) = l {
                            return Ok(Preflight::Completed((Wrapped(Either::Left(output)),)));
                        } else {
                            State::Left(left)
                        }
                    }

                    (Err(..), Ok(r)) => {
                        if let Preflight::Completed(output) = r {
                            return Ok(Preflight::Completed((Wrapped(Either::Right(output)),)));
                        } else {
                            State::Right(right)
                        }
                    }

                    (Err(..), Err(..)) => {
                        // FIXME: appropriate error handling
                        return Err(http::StatusCode::NOT_FOUND.into());
                    }
                }
            }
            _ => panic!("unexpected condition"),
        };

        Ok(Preflight::Incomplete)
    }

    #[inline]
    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Self::Error> {
        match self.state {
            State::Left(ref mut t) => t
                .poll_action(cx)
                .map(|t| t.map(|t| (Wrapped(Either::Left(t)),)))
                .map_err(Into::into),
            State::Right(ref mut t) => t
                .poll_action(cx)
                .map(|t| t.map(|t| (Wrapped(Either::Right(t)),)))
                .map_err(Into::into),
            _ => panic!("unexpected condition"),
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
