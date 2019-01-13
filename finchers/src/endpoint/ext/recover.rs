use {
    crate::{
        action::{
            ActionContext, //
            EndpointAction,
            Preflight,
            PreflightContext,
        },
        endpoint::{Endpoint, IsEndpoint},
        error::Error,
    },
    futures::{Future, IntoFuture, Poll},
};

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub struct Recover<E, F> {
    pub(super) endpoint: E,
    pub(super) f: F,
}

impl<E: IsEndpoint, F> IsEndpoint for Recover<E, F> {}

impl<E, F, Bd, R> Endpoint<Bd> for Recover<E, F>
where
    E: Endpoint<Bd, Output = (R::Item,)>,
    F: Fn(Error) -> R + Clone,
    R: IntoFuture,
    R::Error: Into<Error>,
{
    type Output = (R::Item,);
    type Action = RecoverAction<E::Action, R::Future, F>;

    fn action(&self) -> Self::Action {
        RecoverAction {
            action: self.endpoint.action(),
            f: self.f.clone(),
            in_flight: None,
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct RecoverAction<Act, Fut, F> {
    action: Act,
    f: F,
    in_flight: Option<Fut>,
}

impl<Act, F, Bd, R> EndpointAction<Bd> for RecoverAction<Act, R::Future, F>
where
    Act: EndpointAction<Bd, Output = (R::Item,)>,
    F: Fn(Error) -> R,
    R: IntoFuture,
    R::Error: Into<Error>,
{
    type Output = (R::Item,);

    fn preflight(
        &mut self,
        cx: &mut PreflightContext<'_>,
    ) -> Result<Preflight<Self::Output>, Error> {
        debug_assert!(self.in_flight.is_none());
        self.action.preflight(cx).or_else(|err| {
            self.in_flight = Some((self.f)(err).into_future());
            Ok(Preflight::Incomplete)
        })
    }

    fn poll_action(&mut self, cx: &mut ActionContext<'_, Bd>) -> Poll<Self::Output, Error> {
        loop {
            if let Some(ref mut in_flight) = self.in_flight {
                return in_flight
                    .poll()
                    .map(|x| x.map(|out| (out,)))
                    .map_err(Into::into);
            }
            match self.action.poll_action(cx) {
                Ok(x) => return Ok(x),
                Err(err) => self.in_flight = Some((self.f)(err).into_future()),
            }
        }
    }
}
