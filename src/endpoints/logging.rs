//! Wrapper for logging.

use futures_core::future::{Future, TryFuture};
use futures_core::task;
use futures_core::task::Poll;
use futures_util::try_ready;

use pin_utils::unsafe_pinned;
use std::pin::PinMut;

use http::Response;
use http::StatusCode;
use log::info;
use std::time::Instant;

use crate::endpoint::wrapper::Wrapper;
use crate::endpoint::{Context, Endpoint, EndpointResult};
use crate::error::Error;
use crate::error::Never;
use crate::input::{with_get_cx, Input};
use crate::output::payload::Payload;
use crate::output::{Output, OutputContext};

#[allow(missing_docs)]
pub fn logging() -> Logging<impl Fn(Info<'_>) + Copy + Clone> {
    logging_fn(|info: Info<'_>| {
        info!(
            "{} {} -> {} ({:?})",
            info.input.method(),
            info.input.uri(),
            info.status,
            info.start.elapsed()
        );
    })
}

#[allow(missing_docs)]
pub fn logging_fn<F>(f: F) -> Logging<F> {
    Logging { f }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Logging<F> {
    f: F,
}

impl<'a, E, F> Wrapper<'a, E> for Logging<F>
where
    E: Endpoint<'a>,
    E::Output: Output,
    F: Fn(Info<'_>) + 'a,
{
    type Output = (LoggedResponse<<E::Output as Output>::Body>,);
    type Endpoint = WithLogging<E, F>;

    fn wrap(self, endpoint: E) -> Self::Endpoint {
        WithLogging {
            endpoint,
            f: self.f,
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct WithLogging<E, F> {
    endpoint: E,
    f: F,
}

impl<'a, E, F> Endpoint<'a> for WithLogging<E, F>
where
    E: Endpoint<'a>,
    E::Output: Output,
    F: Fn(Info<'_>) + 'a,
{
    type Output = (LoggedResponse<<E::Output as Output>::Body>,);
    type Future = WithLoggingFuture<'a, E::Future, F>;

    fn apply(&'a self, cx: &mut Context<'_>) -> EndpointResult<Self::Future> {
        let start = Instant::now();
        let future = self.endpoint.apply(cx)?;
        Ok(WithLoggingFuture {
            future,
            f: &self.f,
            start,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct WithLoggingFuture<'a, Fut, F: 'a> {
    future: Fut,
    f: &'a F,
    start: Instant,
}

impl<'a, Fut, F> WithLoggingFuture<'a, Fut, F> {
    unsafe_pinned!(future: Fut);
}

impl<'a, Fut, F> Future for WithLoggingFuture<'a, Fut, F>
where
    Fut: TryFuture<Error = Error>,
    Fut::Ok: Output,
    F: Fn(Info<'_>) + 'a,
{
    type Output = Result<(LoggedResponse<<Fut::Ok as Output>::Body>,), Error>;

    fn poll(mut self: PinMut<'_, Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let x = try_ready!(self.future().try_poll(cx));

        let response = match with_get_cx(|input| {
            let mut ocx = OutputContext::new(input);
            x.respond(&mut ocx)
        }) {
            Ok(response) => response,
            Err(err) => return Poll::Ready(Err(err.into())),
        };

        with_get_cx(|mut input| {
            (self.f)(Info {
                status: response.status(),
                start: self.start,
                input: input.reborrow(),
                _priv: (),
            });
        });

        Poll::Ready(Ok((LoggedResponse(response),)))
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct LoggedResponse<Bd>(Response<Bd>);

impl<Bd: Payload> Output for LoggedResponse<Bd> {
    type Body = Bd;
    type Error = Never;

    #[inline(always)]
    fn respond(self, _: &mut OutputContext<'_>) -> Result<Response<Self::Body>, Self::Error> {
        Ok(self.0)
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Info<'a> {
    pub status: StatusCode,
    pub start: Instant,
    pub input: PinMut<'a, Input>,
    _priv: (),
}
