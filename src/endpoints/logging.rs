//! Wrapper for logging.

use http::Response;
use http::StatusCode;
use std::time::Instant;

use endpoint::wrapper::Wrapper;
use endpoint::{Context, Endpoint, EndpointResult};
use error::Error;
use error::Never;
use input::{with_get_cx, Input};
use output::payload::Payload;
use output::{Output, OutputContext};

/// Create a wrapper for creating an endpoint which dumps log
/// after resolving the future.
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

/// Create a wrapper for creating an endpoint which dumps a log
/// using the specified function.
pub fn logging_fn<F>(f: F) -> Logging<F>
where
    F: Fn(Info<'_>),
{
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

impl<'a, Fut, F> ::futures::Future for WithLoggingFuture<'a, Fut, F>
where
    Fut: ::futures::Future<Error = Error>,
    Fut::Item: Output,
    F: Fn(Info<'_>) + 'a,
{
    type Item = (LoggedResponse<<Fut::Item as Output>::Body>,);
    type Error = Error;

    fn poll(&mut self) -> ::futures::Poll<Self::Item, Self::Error> {
        let x = try_ready!(self.future.poll());

        let response = match with_get_cx(|input| {
            let mut ocx = OutputContext::new(input);
            x.respond(&mut ocx)
        }) {
            Ok(response) => response,
            Err(err) => return Err(err.into()),
        };

        with_get_cx(|input| {
            (self.f)(Info {
                status: response.status(),
                start: self.start,
                input,
                _priv: (),
            });
        });

        Ok((LoggedResponse(response),).into())
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
    pub input: &'a mut Input,
    _priv: (),
}
